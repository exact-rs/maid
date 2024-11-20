use crate::parse;
use crate::structs::{DisplayTask, Maidfile};
use maid::log::prelude::*;

pub fn merge(path: &String) -> Maidfile {
    let mut values = parse::file::read_maidfile(path);
    let imported_values = parse::import::push(values.import.clone());

    for import in imported_values.iter() {
        values = match merge_struct::merge(&values, &import) {
            Ok(merge) => merge,
            Err(err) => error!(%err, "Unable to import tasks"),
        };
    }

    return values;
}

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
