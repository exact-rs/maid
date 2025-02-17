use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use global_placeholders::global;
use macros_rs::fs::folder_exists;
use maid::log::prelude::*;
use std::{fs::write, fs::File, path::PathBuf};
use tar::{Archive, Builder};
use uuid::Uuid;

fn append_to_tar(builder: &mut Builder<GzEncoder<File>>, path: &String) -> Result<(), std::io::Error> {
    let pathbuf = PathBuf::from(path);

    if pathbuf.is_file() {
        builder.append_path(&pathbuf)?;
    } else if pathbuf.is_dir() {
        builder.append_dir_all(&pathbuf, &pathbuf)?;
    }
    Ok(())
}

pub fn remove_tar(file: &String) {
    if let Err(_) = std::fs::remove_file(file) {
        error!("Unable to remove temporary archive. does it exist?");
    }
}

pub fn read_tar(archive: &Vec<u8>) -> Result<String, std::io::Error> {
    if !folder_exists!(&global!("maid.temp_dir")) {
        std::fs::create_dir_all(global!("maid.temp_dir")).unwrap();
        debug!("created maid temp dir");
    }

    let file_name = format!("{}/{}.tgz", global!("maid.temp_dir"), Uuid::new_v4());
    write(&file_name, archive)?;

    Ok(file_name)
}

pub fn unpack_tar(path: &String) -> std::io::Result<()> {
    let archive = File::open(&path)?;
    let tar = GzDecoder::new(archive);
    let mut archive = Archive::new(tar);

    archive.unpack(".")
}

pub fn write_tar(files: &Vec<String>) -> Result<String, std::io::Error> {
    if !folder_exists!(&global!("maid.temp_dir")) {
        std::fs::create_dir_all(global!("maid.temp_dir")).unwrap();
        debug!("created maid temp dir");
    }

    let file_name = format!("{}/{}.tgz", global!("maid.temp_dir"), Uuid::new_v4());
    let archive = File::create(&file_name)?;
    let enc = GzEncoder::new(archive, Compression::default());
    let mut tar = Builder::new(enc);

    debug!("compressing to {}", &file_name);
    for path in files {
        append_to_tar(&mut tar, path)?;
        debug!("{} {:?}", maid::colors::ADD, path);
    }

    Ok(file_name)
}
