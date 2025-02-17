use maid::{
    helpers,
    log::prelude::*,
    models::client::{Runner, Task, UpdateData},
};

use inquire::Text;
use macros_rs::fs::file_exists;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use reqwest::blocking;
use std::{fs::File, io::Write, path::Path, time::Duration};

// rewrite
pub(crate) fn watch(path: &Path) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

    debouncer.watcher().watch(path, RecursiveMode::Recursive).unwrap();
    for events in rx {
        if let Ok(event) = events {
            println!("{:?}", event);
        }
    }
}

pub(crate) fn check_update() {
    let checker = match blocking::get("https://api.maid.ci/versions/latest") {
        Ok(res) => res.json::<UpdateData>(),
        Err(err) => error!(%err, "Unable to check for updates"),
    };

    let version = match checker {
        Ok(body) => body.version,
        Err(err) => error!(%err, "Unable to check for updates"),
    };

    if version == env!("CARGO_PKG_VERSION") {
        info!("Maid is currently on the latest version")
    } else {
        warn!("Your install is currently out of date.\n\nThe current version is {version}\nPlease update using `maid upgrade`")
    }
}

pub(crate) fn clean() {
    if let Ok(_) = std::fs::remove_dir_all(".maid/temp") {
        info!("Purged temp archives")
    }

    match std::fs::remove_dir_all(".maid/cache") {
        Ok(_) => info!("Emptied build cache"),
        Err(_) => warn!("Build cache does not exist, cannot remove"),
    };
}

// improve
pub(crate) fn init() {
    fn create_error(name: &str, path: &str) {
        std::fs::remove_file(path).unwrap();
        error!("An error happened when asking for {}, try again later.", name);
    }

    let path = "maidfile";
    let example_maidfile = "[tasks.example]\ninfo = \"this is a comment\"\nscript = \"echo 'hello world'\"";

    if !file_exists!(path) {
        println!("This utility will walk you through creating a maidfile.\n");

        let mut file = File::create(&path).unwrap();
        let current_dir = std::env::current_dir().unwrap();
        writeln!(&mut file, "[project]").unwrap();

        let name = Text::new("project name:").with_default(&current_dir.file_name().unwrap().to_str().unwrap().to_string()).prompt();
        let version = Text::new("version:").with_default("1.0.0").prompt();

        match name {
            Ok(name) => writeln!(&mut file, "name = \"{name}\"").unwrap(),
            Err(_) => create_error("project name", &path),
        }
        match version {
            Ok(version) => writeln!(&mut file, "version = \"{version}\"").unwrap(),
            Err(_) => create_error("version", &path),
        }

        writeln!(&mut file, "\n{example_maidfile}").unwrap();
        println!("{}", "\n✨ success, saved maidfile".yellow());
        if file_exists!(".git") {
            println!("{}", "dont forget to add '.maid' to your .gitignore".white());
        }
    } else {
        println!("{}", "maidfile already exists, aborting".yellow())
    }
}

pub(crate) fn task(task: Task<toml::Value>) {
    let mut script: Vec<String> = Vec::new();

    if let Some(cmd) = task.script.as_str() {
        script.push(cmd.to_string());
    } else if let Some(array) = task.script.as_array() {
        for value in array {
            match value.as_str() {
                Some(cmd) => script.push(cmd.to_string()),
                None => error!("Unable to parse Maidfile. Missing string value."),
            }
        }
    } else {
        helpers::status::error(task.script.type_str());
    }

    super::script::run_wrapped(Runner {
        script,
        dep: task.dep,
        name: task.name,
        path: task.path,
        args: task.args,
        silent: task.silent,
        project: task.project,
        maidfile: task.maidfile,
    });
}
