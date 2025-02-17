use crate::parse;
use crate::server;

use maid::{
    helpers,
    log::prelude::*,
    models::{
        client::{ConnectionData, ConnectionInfo, Kind, Level, Task, Websocket},
        shared::Maidfile,
    },
};

use macros_rs::fmt::fmtstr;
use reqwest::blocking::Client;
use toml::Value;
use tungstenite::protocol::frame::{coding::CloseCode::Normal, CloseFrame};
use tungstenite::{client::connect_with_config, client::IntoClientRequest, protocol::WebSocketConfig, Message};

fn health(client: Client, values: Maidfile<Value>) -> server::api::health::Route {
    let address = server::parse::address(&values);
    let token = server::parse::token(&values);

    let response = match client.get(fmtstr!("{address}/api/health")).header("Authorization", fmtstr!("Bearer {token}")).send() {
        Ok(res) => res,
        Err(err) => error!(%err, "Unable to connect to the maid server. Is it up?"),
    };

    let body = match response.json::<server::api::health::Route>() {
        Ok(body) => body,
        Err(err) => error!(%err, "Unable to connect to the maid server. Is the token correct?"),
    };

    return body;
}

pub fn connect(path: &String) {
    let values = parse::merge(path);
    let client = Client::new();
    let body = health(client, values);

    println!(
        "{}\n{}\n{}\n{}",
        "Server Info".green().bold(),
        format!(" {}: {}", "- Version".white(), body.version.data.color(body.version.hue)),
        format!(" {}: {}", "- Platform".white(), body.platform.data.color(body.platform.hue)),
        format!(" {}: {}", "- Engine".white(), body.engine.data.color(body.engine.hue)),
    );

    println!(
        "{}\n{}\n{}\n{}",
        "Server Status".green().bold(),
        format!(" {}: {}", "- Uptime".white(), body.status.uptime.data.color(body.status.uptime.hue)),
        format!(" {}: {}", "- Healthy".white(), body.status.healthy.data.color(body.status.healthy.hue)),
        format!(" {}: {}", "- Containers".white(), format!("{:?}", body.status.containers.data).color(body.status.containers.hue)),
    );
}

pub fn remote(task: Task<Value>) {
    let mut script: Vec<&str> = vec![];

    if task.script.is_str() {
        match task.script.as_str() {
            Some(cmd) => script.push(cmd),
            None => error!("Unable to parse Maidfile. Missing string value."),
        };
    } else if task.script.is_array() {
        match IntoIterator::into_iter(match task.script.as_array() {
            Some(iter) => iter,
            None => error!("Unable to parse Maidfile. Missing array value."),
        }) {
            mut iter => loop {
                match Iterator::next(&mut iter) {
                    Some(val) => match val.as_str() {
                        Some(cmd) => script.push(cmd),
                        None => error!("Unable to parse Maidfile. Missing string value."),
                    },
                    None => break,
                };
            },
        }
    } else {
        helpers::status::error(task.script.type_str())
    }

    let client = Client::new();
    let body = health(client, task.maidfile.clone());
    let (_, websocket, token, host, port) = server::parse::all(task.maidfile.clone());

    crate::log!(Level::Info, "connecting to {host}:{port}");

    if body.status.healthy.data == "yes" {
        crate::log!(Level::Notice, "server reports healthy");
    } else {
        crate::log!(Level::Warning, "failed to connect");
    }

    let websocket_config = WebSocketConfig {
        max_frame_size: Some(314572800),
        ..Default::default()
    };

    let mut request = websocket.into_client_request().expect("Can't connect");
    request.headers_mut().insert("Authorization", fmtstr!("Bearer {token}").parse().unwrap());

    let (mut socket, response) = connect_with_config(request, Some(websocket_config), 3).expect("Can't connect");
    debug!("response code: {}", response.status());

    let connection_data = ConnectionData {
        info: ConnectionInfo {
            name: task.name.clone(),
            args: task.args.clone(),
            remote: task.remote.clone().unwrap(),
            script: script.clone().iter().map(|&s| s.to_string()).collect(),
        },
        maidfile: task.maidfile.clone(),
    };

    let file_name = match server::file::write_tar(&task.remote.unwrap().push) {
        Ok(name) => name,
        Err(err) => error!(%err, "Unable to create archive"),
    };

    debug!("sending information");
    socket.send(Message::Text(serde_json::to_string(&connection_data).unwrap())).unwrap();

    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                if let Ok(Websocket { message, kind, level, .. }) = serde_json::from_str::<Websocket>(&text) {
                    match kind {
                        Kind::Done => break,
                        Kind::Message => crate::log!(level, "{}", message.unwrap()),
                        Kind::Binary => socket.send(Message::Binary(std::fs::read(&file_name).unwrap())).unwrap(),
                    }
                }
            }
            Ok(Message::Binary(archive)) => {
                let archive_name = match server::file::read_tar(&archive) {
                    Ok(name) => name,
                    Err(err) => error!(%err, "Unable to read archive"),
                };

                if let Err(err) = server::file::unpack_tar(&archive_name) {
                    error!(%err, "Unable to create archive")
                }

                server::file::remove_tar(&archive_name);
            }
            Err(err) => {
                crate::log!(Level::Fatal, "{err}");
                break;
            }
            _ => (),
        };
    }

    server::file::remove_tar(&file_name);
    // run.rs:96 implement that later
    println!("\n{} {}", maid::colors::OK, "finished task successfully".bright_green());
    println!("{}", "removed temporary archive".bright_magenta());

    if let Err(err) = socket.close(Some(CloseFrame {
        code: Normal,
        // run.rs:96 implement that later
        reason: std::borrow::Cow::Borrowed("finished task successfully"),
    })) {
        error!(%err, "Unable to close socket")
    };
}
