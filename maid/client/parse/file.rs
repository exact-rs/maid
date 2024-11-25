use crate::structs::Maidfile;
use maid::log::prelude::*;

use macros_rs::{crashln, string, then};
use std::{env, fs, io::Result, path::Path, path::PathBuf};

macro_rules! create_path {
    ($file_name:expr, $kind:expr) => {{
        let mut file_path = PathBuf::new();
        file_path.push($file_name);
        file_path.set_extension($kind);
        file_path
    }};
}

#[derive(Debug)]
struct Filesystem {
    path: Option<PathBuf>,
    is_file: bool,
}

fn working_dir() -> PathBuf {
    match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            crashln!("Unable to find current working dir");
        }
    }
}

#[allow(unused_variables)]
fn find_path(path: &Path, file_name: &str, kind: &str) -> Result<Option<fs::DirEntry>> {
    #[cfg(target_os = "linux")]
    {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = Box::leak(create_path!(file_name, kind).into_boxed_path()).to_string_lossy().to_string();

            if entry.file_name().to_string_lossy().eq_ignore_ascii_case(&file_path) {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(None)
    }
}

fn find_file(starting_directory: &Path, file_name: &String) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let find_kind = |kind: &str, mut inner: PathBuf| -> Filesystem {
        let file_path = create_path!(file_name, kind);
        then!(working_dir() != starting_directory, inner.pop());

        match find_path(starting_directory, file_name, kind).unwrap() {
            Some(file) => inner.push(file.path()),
            None => inner.push(file_path),
        }

        Filesystem {
            path: Some(inner.clone()),
            is_file: inner.is_file(),
        }
    };

    loop {
        for extension in vec!["", "toml", "yaml", "yml", "json", "hcl"].iter() {
            let kind = find_kind(extension, path.clone());
            then!(kind.is_file, return kind.path);
        }
        then!(!path.pop(), break);
    }

    return None;
}

fn read_file(path: PathBuf, kind: &str) -> Maidfile {
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            warn!("{}", err);
            crashln!("Cannot find maidfile. Does it exist?");
        }
    };

    let result = match kind {
        "toml" => toml::from_str(&contents).map_err(|err| string!(err)),
        "json" => serde_json::from_str(&contents).map_err(|err| string!(err)),
        "hcl" => hcl::from_str(&contents).map_err(|err| string!(err)),
        "yaml" | "yml" => serde_yaml::from_str(&contents).map_err(|err| string!(err)),
        _ => error!("Invalid format, cannot read Maidfile"),
    };

    match result {
        Ok(parsed) => parsed,
        Err(err) => error!("Cannot read Maidfile.\n{}", err.white()),
    }
}

pub fn read_maidfile_with_error(filename: &String, error: &str) -> Maidfile {
    match env::current_dir() {
        Ok(path) => match find_file(&path, &filename) {
            Some(path) => {
                let extension = path.extension().and_then(|s| s.to_str());
                debug!(path = path.display().to_string(), kind = extension, "Found tasks");

                match extension {
                    Some("yaml") | Some("yml") | Some("json") | Some("hcl") => read_file(path.clone(), extension.unwrap()),
                    _ => read_file(path, "toml"),
                }
            }
            None => error!("{error}"),
        },
        Err(err) => error!(%err, "Home directory could not found"),
    }
}

pub fn find_maidfile_root(filename: &String) -> PathBuf {
    match env::current_dir() {
        Ok(path) => match find_file(&path, &filename) {
            Some(mut path) => {
                path.pop();
                debug!("Found project path: {}", path.display());
                return path;
            }
            None => error!("Cannot find project root."),
        },
        Err(err) => error!(%err, "Home directory could not found"),
    }
}

pub fn read_maidfile(filename: &String) -> Maidfile { read_maidfile_with_error(filename, "Cannot find maidfile. Does it exist?") }
