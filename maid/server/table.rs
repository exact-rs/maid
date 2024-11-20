use crate::helpers;
use crate::structs::Maidfile;
use maid::log::prelude::*;

use macros_rs::{str, ternary};
use serde_json::Value;
use std::path::PathBuf;
use std::{collections::BTreeMap, collections::HashMap, env};
use text_placeholder::Template;

pub fn create(values: Maidfile, args: &Vec<String>, project: PathBuf) -> HashMap<&str, &str> {
    let mut table = HashMap::new();
    let empty_env: BTreeMap<String, Value> = BTreeMap::new();

    table.insert("os.platform", env::consts::OS);
    table.insert("os.arch", env::consts::ARCH);

    trace!(os_platform = env::consts::OS);
    trace!(os_arch = env::consts::ARCH);

    match env::current_dir() {
        Ok(path) => {
            table.insert("dir.current", helpers::string::path_to_str(&path));
            trace!(dir_current = path.display().to_string());
        }
        Err(err) => error!(%err, "Current directory could not be added as script variable."),
    }

    match home::home_dir() {
        Some(path) => {
            table.insert("dir.home", helpers::string::path_to_str(&path));
            trace!(dir_home = path.display().to_string());
        }
        None => error!("Home directory could not be added as script variable."),
    }

    table.insert("dir.project", helpers::string::path_to_str(&project));
    trace!(dir_project = project.display().to_string());

    for (pos, arg) in args.iter().enumerate() {
        trace!("arg_{pos} = \"{arg}\"");
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

        env::set_var(key, value_formatted.clone());
        trace!("env_{key} = \"{value_formatted}\"");
        table.insert(str!(format!("env.{}", key.clone())), str!(value_formatted));
    }

    return table;
}
