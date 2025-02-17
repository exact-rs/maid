use crate::models::shared::{Maidfile, Remote};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    pub target: Vec<String>,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task<T> {
    pub maidfile: Maidfile<T>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<Remote>,
    pub project: PathBuf,
    pub script: T,
    pub path: String,
    pub args: Vec<String>,
    pub silent: bool,
    pub dep: Dependency,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dependency {
    pub active: bool,
    pub verbose: bool,
}

#[derive(Clone, Debug)]
pub struct Runner<T> {
    pub maidfile: Maidfile<T>,
    pub name: String,
    pub script: Vec<String>,
    pub path: String,
    pub args: Vec<String>,
    pub project: PathBuf,
    pub silent: bool,
    pub dep: Dependency,
}

#[derive(Debug)]
pub struct DisplayTask {
    pub name: String,
    pub formatted: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Level {
    None,
    Fatal,
    Docker,
    Debug,
    Error,
    Notice,
    Info,
    Build,
    Warning,
    Success,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Kind {
    Done,
    Binary,
    Message,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Websocket {
    pub level: Level,
    pub kind: Kind,
    pub time: i64,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionInfo {
    pub name: String,
    pub remote: Remote,
    pub args: Vec<String>,
    pub script: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionData<T> {
    pub info: ConnectionInfo,
    pub maidfile: Maidfile<T>,
}

#[derive(Deserialize)]
pub struct UpdateData {
    pub version: String,
    pub download: String,
}
