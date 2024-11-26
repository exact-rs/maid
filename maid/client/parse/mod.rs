pub mod file;
pub mod import;

use maid::log::prelude::*;
use maid::models::shared::Maidfile;
use toml::Value;

pub(crate) fn merge(path: &String) -> Maidfile<Value> {
    let mut values = file::read_maidfile(path);
    let imported_values = import::push(values.import.clone());

    for import in imported_values.iter() {
        values = match merge_struct::merge(&values, &import) {
            Ok(merge) => merge,
            Err(err) => error!(%err, "Unable to import tasks"),
        };
    }

    return values;
}
