use macros_rs::{exp::ternary, fmt::string};
use maid::models::shared::Maidfile;
use toml::Value;

pub fn address(values: &Maidfile<Value>) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => {
                let prefix = ternary!(server.address.tls, "https", "http");
                format!("{}://{}:{}", prefix, server.address.host, server.address.port)
            }
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn websocket(values: &Maidfile<Value>) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => {
                let prefix = ternary!(server.address.tls, "wss", "ws");
                format!("{}://{}:{}/ws/gateway", prefix, server.address.host, server.address.port)
            }
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn host(values: &Maidfile<Value>) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.address.host.clone(),
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn port(values: &Maidfile<Value>) -> i64 {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.address.port.clone(),
            None => 0,
        },
        None => 0,
    }
}

pub fn token(values: &Maidfile<Value>) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.token.clone(),
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn all(maidfile: Maidfile<Value>) -> (String, String, String, String, i64) { (address(&maidfile), websocket(&maidfile), token(&maidfile), host(&maidfile), port(&maidfile)) }
