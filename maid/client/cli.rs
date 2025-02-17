pub(crate) mod dispatch;
pub(crate) mod script;
pub(crate) mod tasks;

use crate::{parse, server, task};

use maid::{
    helpers,
    log::prelude::*,
    models::{
        client::{CacheConfig, Dependency, Task},
        shared::{Cache, Project},
    },
};

use fs_extra::dir::get_size;
use global_placeholders::global;
use human_bytes::human_bytes;
use std::{env, fs, path::Path, time::Instant};

use macros_rs::{
    exp::ternary,
    fmt::{fmtstr, string},
    fs::{file_exists, folder_exists},
};

pub(crate) fn get_version(short: bool) -> String {
    if short {
        format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!("{} ({} {})", env!("CARGO_PKG_VERSION"), env!("GIT_HASH"), env!("BUILD_DATE"))
    }
}

pub(crate) fn info(path: &String) {
    let values = parse::merge(path).project;
    let project_root = parse::file::find_maidfile_root(path);

    let project_name = match values.to_owned() {
        Some(project) => project.name,
        None => Project::default().name,
    };

    let project_version = match values.to_owned() {
        Some(project) => project.version,
        None => Project::default().version,
    };

    match project_name {
        Some(name) => info!("Project {name} info\n"),
        None => info!("Project info\n"),
    };

    match project_version {
        Some(version) => println!(
            "{}\n{}",
            format!("{}: {}", "Version".white(), version.bright_yellow()),
            format!("{}: {}", "Directory".white(), project_root.to_string_lossy().bright_yellow())
        ),
        None => println!("{}", format!("{}: {}", "Directory".white(), project_root.to_string_lossy().bright_yellow())),
    };
}

pub(crate) fn env(path: &String) {
    let values = parse::merge(path);

    let project_name = match values.project {
        Some(project) => project.name,
        None => Project::default().name,
    };

    if let Some(env) = values.env {
        match project_name {
            Some(name) => info!("ENV for {name}\n"),
            None => info!("ENV for this project\n"),
        };

        for (key, value) in env {
            println!("{}{}{value}", key.bright_cyan(), "=".white())
        }
    } else {
        error!("No ENV values defined for this project")
    }
}

pub(crate) fn exec(task: &str, args: &Vec<String>, path: &String, silent: bool, is_dep: bool, is_remote: bool, log_level: Option<tracing::Level>, force: bool, log_deps: bool) {
    debug!("Starting maid {}", env!("CARGO_PKG_VERSION"));

    if task.is_empty() {
        if is_remote {
            tasks::list_remote(path, silent, log_level);
        } else {
            tasks::list_all(path, silent, log_level, force);
        }
    } else {
        let values = parse::merge(path);
        let project_root = parse::file::find_maidfile_root(path);
        let cwd = &helpers::file::get_current_working_dir();

        if values.tasks.get(task).is_none() {
            error!("Could not find the task '{task}'. Does it exist?");
        }

        if is_remote && values.tasks.get(task).unwrap().remote.is_none() {
            error!("Could not find the remote task '{task}'. Does it exist?");
        }

        match values.tasks.get(task).unwrap().remote.as_ref() {
            Some(val) => {
                if val.exclusive && !is_remote {
                    error!("Task '{task}' is remote only.");
                }
            }
            None => {}
        }

        if !is_remote {
            match &values.tasks[task].depends {
                Some(deps) => {
                    let start = Instant::now();
                    let ticks = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let template = fmtstr!("{{prefix:.white}} {{spinner:.yellow}}{{msg}} {}", "({elapsed})".bright_cyan());
                    let pb = task::progress::init(ticks, template, 80);

                    for (index, item) in deps.iter().enumerate() {
                        let is_verbose_dep = item.starts_with("log:");
                        let name = item.strip_prefix("log:").unwrap_or(item.as_str());

                        pb.set_prefix(format!("[{}/{}]", index + 1, deps.len()));
                        pb.set_message(fmtstr!("{} {name}", "running dependency".bright_yellow()));

                        exec(&name, args, path, true, true, is_remote, log_level, force, is_verbose_dep);
                    }

                    if !is_dep {
                        task::progress::finish();
                        println!(
                            "{} {} in {} {}\n",
                            maid::colors::OK,
                            format!("finished {} {}", deps.len(), ternary!(deps.len() > 1, "dependencies", "dependency")).bright_green(),
                            format!("{:.2?}", start.elapsed()).yellow(),
                            format!("[{}]", deps.join(", ")).white()
                        )
                    }
                }
                None => {}
            };
        }

        let cache = match &values.tasks[task].cache {
            Some(cache) => cache.clone(),
            None => Cache { path: string!(""), target: vec![] },
        };

        let task_path = match &values.tasks[task].path {
            Some(path) => ternary!(path == "", helpers::string::path_to_str(project_root.as_path()), ternary!(path == "%{dir.current}", cwd, path)),
            None => helpers::string::path_to_str(project_root.as_path()),
        }
        .to_string();

        if !cache.path.trim().is_empty() && !cache.target.is_empty() && !is_remote {
            if !folder_exists!(&global!("maid.cache_dir", task)) {
                fs::create_dir_all(global!("maid.cache_dir", task)).unwrap();
                debug!("created maid cache dir");
            }

            let hash = task::cache::create_hash(&cache.path);
            let config_path = format!(".maid/cache/{task}/{}.toml", task);

            if !file_exists!(&config_path) {
                match fs::write(
                    config_path.clone(),
                    toml::to_string(&CacheConfig {
                        target: cache.target.clone(),
                        hash: string!(""),
                    })
                    .unwrap(),
                ) {
                    Ok(_) => debug!("Created {task} cache config"),
                    Err(err) => error!(%err, "Cannot create cache config"),
                };
            }

            let contents = match fs::read_to_string(config_path.clone()) {
                Ok(content) => content,
                Err(err) => error!(%err, "Cannot read cache config"),
            };

            let json = match toml::from_str::<CacheConfig>(&contents) {
                Ok(contents) => contents,
                Err(err) => error!(%err, "Cannot read cache config"),
            };

            if json.hash == hash && !is_dep && !force {
                println!("{}", "skipping task due to cached files".bright_magenta());

                for target in cache.target.clone() {
                    let cache_file = format!(".maid/cache/{task}/target/{}", Path::new(&target.clone()).file_name().unwrap().to_str().unwrap());

                    println!(
                        "{} ({})",
                        format!("copied target '{}' from cache", target.clone()).magenta(),
                        format!("{}", human_bytes(get_size(cache_file.clone()).unwrap_or_default() as f64).white())
                    );

                    match fs::copy(Path::new(&cache_file), target.to_owned()) {
                        Ok(_) => debug!("copied target file {}", target),
                        Err(_) => {
                            println!("");
                            match fs::remove_dir_all(format!(".maid/cache/{task}")) {
                                Ok(_) => warn!("Cannot copy target file, rebuilt build cache"),
                                Err(_) => error!("Build cache does not exist, cannot remove"),
                            }
                        }
                    };
                }

                std::process::exit(0);
            } else {
                match fs::write(
                    config_path.clone(),
                    toml::to_string(&CacheConfig {
                        target: cache.target,
                        hash: hash.to_owned(),
                    })
                    .unwrap(),
                ) {
                    Ok(_) => debug!("added hash for {task} -> {hash}"),
                    Err(err) => error!(%err, "error creating cache config"),
                };
            }
        };

        debug!("Is remote?: {is_remote}");
        debug!("Project dir: {:?}", project_root);
        debug!("Task path: {task_path}");
        debug!("Working dir: {cwd}");
        debug!("Started task: {task}");

        if !silent && !is_remote {
            ternary!(
                task_path == helpers::string::path_to_str(project_root.as_path()) || task_path == "%{dir.current}" || task_path == "." || task_path == *cwd,
                println!("{} {}", maid::colors::ARROW, &values.tasks[task].script),
                println!("{} {} {}", format!("({task_path})").bright_cyan(), maid::colors::ARROW, &values.tasks[task].script)
            )
        }

        if is_remote {
            server::cli::remote(Task {
                silent,
                maidfile: values.clone(),
                name: string!(task),
                project: project_root,
                remote: values.tasks[task].remote.clone(),
                script: values.tasks[task].script.clone(),
                path: task_path.clone(),
                args: args.clone(),
                dep: Dependency { active: is_dep, verbose: log_deps },
            });
        } else {
            dispatch::task(Task {
                silent,
                maidfile: values.clone(),
                name: string!(task),
                project: project_root,
                remote: None,
                script: values.tasks[task].script.clone(),
                path: task_path.clone(),
                args: args.clone(),
                dep: Dependency { active: is_dep, verbose: log_deps },
            });
        }
    }
}
