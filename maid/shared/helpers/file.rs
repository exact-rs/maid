use crate::log::prelude::*;
use std::env;

pub fn get_current_working_dir() -> String {
    match env::current_dir() {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(err) => error!(%err, "Unable to find current working dir"),
    }
}
