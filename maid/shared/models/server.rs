use crate::models::shared::{Maidfile, Remote};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionInfo {
    pub name: String,
    pub remote: Remote,
    pub args: Vec<String>,
    pub script: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionData {
    pub info: ConnectionInfo,
    pub maidfile: Maidfile<Value>,
}
