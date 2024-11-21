use crate::cli;
use crate::helpers;
use crate::parse;
use crate::structs;
use crate::table;

use inquire::Select;
use macros_rs::{string, ternary};
use maid::log::prelude::*;
use text_placeholder::Template;

pub fn json(path: &String, args: &Vec<String>, hydrate: bool) {
    let values = helpers::maidfile::merge(path);
    let project_root = parse::file::find_maidfile_root(path);
    let json = values.clone().to_json();
    let table = table::create(values.clone(), args, project_root);
    let hydrated_json = Template::new_with_placeholder(&json, "%{", "}").fill_with_hashmap(&table);

    println!("{}", ternary!(hydrate.clone(), hydrated_json, json))
}

pub struct List;
impl List {
    pub fn all(path: &String, silent: bool, log_level: Option<tracing::Level>, force: bool) {
        let values = helpers::maidfile::merge(path);
        let mut options: Vec<_> = values
            .tasks
            .iter()
            .map(|(key, task)| {
                let info = match &task.info {
                    Some(info) => match info.trim().len() < 1 {
                        true => "(no description)".to_string().bright_red(),
                        false => format!("({info})").white(),
                    },
                    None => "(no description)".to_string().bright_red(),
                };

                let verbose = match log_level.unwrap() {
                    tracing::Level::INFO => string!(),
                    _ => string!(task.script),
                };

                let hidden = match key.starts_with("_") {
                    true => true,
                    false => match task.hide {
                        Some(val) => val,
                        None => match task.remote.as_ref() {
                            Some(val) => val.exclusive,
                            None => false,
                        },
                    },
                };

                return structs::DisplayTask {
                    name: key.clone(),
                    formatted: format!("{} {} {}", format!("{key}").bright_yellow(), info, verbose.bright_blue()),
                    hidden: hidden.clone(),
                };
            })
            .collect();

        options.retain(|key| key.hidden == false);
        match Select::new("Select a task to run:", options).prompt() {
            Ok(task) => {
                debug!("Starting {}", task.name);
                cli::exec(&String::from(task.name), &vec![String::from("")], &path, silent, false, false, log_level, force);
            }

            Err(_) => println!("{}", "Aborting...".white()),
        }
    }

    pub fn remote(path: &String, silent: bool, log_level: Option<tracing::Level>) {
        let values = helpers::maidfile::merge(path);
        let mut options: Vec<_> = values
            .tasks
            .iter()
            .map(|(key, task)| {
                let info = match &task.info {
                    Some(info) => match info.trim().len() < 1 {
                        true => "(no description)".to_string().bright_red(),
                        false => format!("({info})").white(),
                    },
                    None => "(no description)".to_string().bright_red(),
                };

                let verbose = match log_level.unwrap() {
                    tracing::Level::INFO => string!(),
                    _ => string!(task.script),
                };

                let hidden = match task.remote {
                    Some(_) => false,
                    None => true,
                };

                return structs::DisplayTask {
                    name: key.clone(),
                    formatted: format!("{} {} {}", format!("{key}").bright_yellow(), info, verbose.bright_blue()),
                    hidden: hidden.clone(),
                };
            })
            .collect();

        options.retain(|key| key.hidden == false);
        match Select::new("Select a remote task to run:", options).prompt() {
            Ok(task) => {
                debug!("Starting {}", task.name);
                cli::exec(&String::from(task.name), &vec![String::from("")], &path, silent, false, true, log_level, false);
            }

            Err(_) => println!("{}", "Aborting...".white()),
        }
    }
}
