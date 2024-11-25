use crate::parse;
use macros_rs::fmt::fmtstr;
use maid::models::client::Maidfile;

pub fn push(path_list: Option<Vec<String>>) -> Vec<Maidfile> {
    let mut values: Vec<Maidfile> = vec![];

    let mut add_values = |paths: Vec<String>| {
        for path in paths.iter() {
            let err = fmtstr!("{} cannot be imported. Does the file exist?", path);
            let value = parse::file::read_maidfile_with_error(path, err);
            values.push(value)
        }
    };

    match path_list {
        Some(paths) => add_values(paths),
        None => {}
    };

    return values;
}
