use crate::helpers;
use crate::log::prelude::*;
use crate::models::shared::Maidfile;

use macros_rs::{exp::ternary, fmt::str};
use std::path::PathBuf;
use std::{collections::BTreeMap, collections::HashMap, env};
use text_placeholder::Template;

pub fn create<T: ToString>(values: Maidfile<T>, args: &Vec<String>, project: PathBuf) -> HashMap<&str, &str> {
    let mut table = HashMap::new();
    let empty_env: BTreeMap<String, T> = BTreeMap::new();

    trace!(value = env::consts::OS, "os.platform");
    trace!(value = env::consts::ARCH, "os.arch");

    table.insert("os.platform", env::consts::OS);
    table.insert("os.arch", env::consts::ARCH);

    match env::current_dir() {
        Ok(path) => {
            trace!(value = path.display().to_string(), "dir.current");
            table.insert("dir.current", helpers::string::path_to_str(&path));
        }
        Err(err) => error!(%err, "Current directory could not be added as script variable."),
    }

    match home::home_dir() {
        Some(path) => {
            trace!(value = path.display().to_string(), "dir.home");
            table.insert("dir.home", helpers::string::path_to_str(&path));
        }
        None => error!("Home directory could not be added as script variable."),
    }

    trace!(value = project.display().to_string(), "dir.project");
    table.insert("dir.project", helpers::string::path_to_str(&project));

    for (pos, arg) in args.iter().enumerate() {
        trace!(value = arg, "arg.{pos}");
        table.insert(str!(format!("arg.{pos}")), arg);
    }

    let user_env = match &values.env {
        Some(env) => env.iter(),
        None => empty_env.iter(),
    };

    for (key, value) in user_env {
        let value_formatted = ternary!(
            value.to_string().starts_with("\""),
            helpers::string::trim_start_end(str!(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))).replace("\"", "\\\""),
            str!(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table)).replace("\"", "\\\"")
        );

        trace!(value = value_formatted, "env.{key}");
        env::set_var(key, value_formatted.clone());
        table.insert(str!(format!("env.{}", key.clone())), str!(value_formatted));
    }

    return table;
}
