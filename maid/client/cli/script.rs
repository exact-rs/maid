use maid::{
    helpers,
    log::prelude::*,
    models::{client::Runner, shared::Cache},
    table,
};

use std::{
    env,
    io::Error,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::Instant,
};

use crate::shell::IntoArgs;
use fs_extra::dir::get_size;
use human_bytes::human_bytes;
use text_placeholder::Template;

pub(crate) fn run_wrapped(runner: Runner<toml::Value>) {
    let start = Instant::now();

    let mut cmd: Child;
    let mut status_array: Vec<Result<ExitStatus, Error>> = vec![];

    for string in runner.script {
        let start = Instant::now();

        let working_dir = runner.project.join(Path::new(&runner.path));
        let table = table::create(runner.maidfile.to_owned(), &runner.args, runner.project.to_owned());
        let script = Template::new_with_placeholder(&string, "%{", "}").fill_with_hashmap(&table);

        let (name, args) = match script.try_into_args() {
            Ok(mut args) => (args.remove(0), args),
            Err(err) => error!(%err, "Script could not be parsed into args"),
        };

        debug!("Original Script: {string}");
        debug!("Parsed Script: {script}");
        debug!("Execute Command: '{name} {}'", args.join(" "));

        match env::set_current_dir(&working_dir) {
            Ok(_) => debug!("Working directory: {working_dir:?}"),
            Err(err) => error!(%err, "Failed to set working directory {working_dir:?}"),
        };

        if runner.dep.active {
            let is_verbose = runner.dep.verbose;

            cmd = match Command::new(&name)
                .stdout(if is_verbose { Stdio::inherit() } else { Stdio::null() })
                .stderr(if is_verbose { Stdio::inherit() } else { Stdio::null() })
                .stdin(if is_verbose { Stdio::inherit() } else { Stdio::null() })
                .args(args.to_owned())
                .spawn()
            {
                Ok(output) => output,
                Err(err) => error!(%err, "Cannot start command {name}."),
            };
        } else {
            cmd = match Command::new(&name)
                .args(args.to_owned())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .spawn()
            {
                Ok(output) => output,
                Err(err) => error!(%err, "Cannot start command {name}."),
            };
        }

        let status = cmd.wait();
        let exit_code = helpers::status::code(&status);

        status_array.push(status);
        debug!("Finished cmd: '{name} {}' with exit code: {:?} in {:.2?}", args.join(" "), exit_code, start.elapsed());
    }

    let status = match status_array.last() {
        Some(status) => status,
        None => error!("Failed to fetch final status code."),
    };

    let cache = match &runner.maidfile.tasks[&runner.name].cache {
        Some(cache) => cache.clone(),
        None => Cache { path: "".to_string(), target: vec![] },
    };

    let exit_code = helpers::status::code(status);
    let success = helpers::status::success(&status);

    if !runner.silent {
        if success {
            println!("\n{} {}", maid::colors::OK, "finished task successfully".bright_green());
            if !cache.path.trim().is_empty() && !cache.target.is_empty() {
                for target in cache.target {
                    let cache_file = format!(".maid/cache/{}/target/{}", runner.name, Path::new(&target).file_name().unwrap().to_str().unwrap());
                    match std::fs::copy(Path::new(&target), cache_file.clone()) {
                        Ok(_) => {
                            println!(
                                "{} ({})",
                                format!("saved target '{}' to cache", target).bright_magenta(),
                                format!("{}", human_bytes(get_size(cache_file.clone()).unwrap() as f64).white())
                            );
                            debug!("saved target file {}", target)
                        }
                        Err(err) => error!(%err, target, "Cannot save target file"),
                    };
                }
            }
            println!("{} took {}", runner.name.white(), format!("{:.2?}", start.elapsed()).yellow());
        } else {
            println!("\n{} {} {}", maid::colors::FAIL, "exited with status code".bright_red(), format!("{}", exit_code).red());
            println!("{} took {}", runner.name.white(), format!("{:.2?}", start.elapsed()).yellow());
        }
    } else {
        if success {
            if !cache.path.trim().is_empty() && !cache.target.is_empty() {
                for target in cache.target {
                    let cache_file = format!(".maid/cache/{}/target/{}", runner.name, Path::new(&target).file_name().unwrap().to_str().unwrap());
                    match std::fs::copy(Path::new(&target), cache_file.clone()) {
                        Ok(_) => println!(
                            "{} {}{}{}",
                            maid::colors::ADD,
                            format!("{}", target).bright_green(),
                            maid::colors::SEP,
                            format!("{}", human_bytes(get_size(cache_file.clone()).unwrap() as f64).bright_cyan())
                        ),
                        Err(err) => warn!(%err, %target, "Cache miss"),
                    };
                }
            }
        }
    }
}
