use maid::log::prelude::*;
use std::{io::Error, process::ExitStatus};

pub fn error(debug_err: &str) {
    error!("Unable to parse maidfile. Contains unexpected {debug_err} values.");
}

pub fn code(status: &Result<ExitStatus, Error>) -> i32 {
    match status.as_ref() {
        Ok(status) => match status.code() {
            Some(iter) => iter,
            None => error!("Missing status value"),
        },
        Err(err) => error!(%err, "Unknown error"),
    }
}

pub fn success(status: &Result<ExitStatus, Error>) -> bool {
    match status.as_ref() {
        Ok(status) => status.success(),
        Err(err) => error!(%err, "Unknown error"),
    }
}
