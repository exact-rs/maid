use crate::log::prelude::*;
use crate::models::client::{DisplayTask, Maidfile};

impl Maidfile {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(contents) => contents,
            Err(err) => error!(%err, "Cannot read Maidfile"),
        }
    }
}

impl std::fmt::Display for DisplayTask {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(&self.formatted, f) }
}
