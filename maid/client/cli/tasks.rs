use crate::{cli, parse};
use maid::{log::prelude::*, models::client::DisplayTask, table};

use inquire::Select;
use text_placeholder::Template;
use tracing::Level;

fn create_options(path: &String, remote: bool, log_level: Option<Level>) -> Vec<DisplayTask> {
    let values = parse::merge(path);
    let mut options: Vec<DisplayTask> = Vec::new();

    for (key, task) in &values.tasks {
        let mut verbose = String::default();
        let mut desc = format!(" {}", "undescribed".on_black().red());

        if let Some(info) = &task.info {
            if !info.trim().is_empty() {
                desc = format!("{} {info}", ":".white())
            }
        }

        if log_level.unwrap_or(Level::INFO) != Level::INFO {
            verbose = task.script.to_string()
        };

        let hidden = match remote {
            true => match task.remote {
                Some(_) => false,
                None => true,
            },
            false => key.starts_with("_") || task.hide.map_or(task.remote.as_ref().map_or(false, |r| r.exclusive), |h| h),
        };

        if !hidden {
            options.push(DisplayTask {
                name: key.to_owned(),
                formatted: format!("{}{desc} {}", format!("{key}").truecolor(255, 165, 0), verbose.bright_blue()),
            });
        }
    }

    options
}

pub(crate) fn list_json(path: &String, args: &Vec<String>, hydrate: bool) {
    let values = parse::merge(path);
    let json = values.to_json();

    if hydrate {
        let project = parse::file::find_maidfile_root(path);
        let table = table::create(values, args, project);
        let hydrated = Template::new_with_placeholder(&json, "%{", "}").fill_with_hashmap(&table);

        println!("{hydrated}")
    } else {
        println!("{json}")
    }
}

pub(crate) fn list_all(path: &String, silent: bool, log_level: Option<Level>, force: bool) {
    let options = create_options(path, false, log_level);

    match Select::new("Select a task to run:", options).prompt() {
        Ok(task) => {
            debug!("Starting {}", task.name);
            cli::exec(&String::from(task.name), &vec![String::from("")], &path, silent, false, false, log_level, force, false);
        }

        Err(_) => println!("{}", "Aborting...".white()),
    }
}

pub(crate) fn list_remote(path: &String, silent: bool, log_level: Option<Level>) {
    let options = create_options(path, true, log_level);

    match Select::new("Select a remote task to run:", options).prompt() {
        Ok(task) => {
            debug!("Starting {}", task.name);
            cli::exec(&String::from(task.name), &vec![String::from("")], &path, silent, false, true, log_level, false, false);
        }

        Err(_) => println!("{}", "Aborting...".white()),
    }
}
