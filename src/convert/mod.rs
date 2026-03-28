use std::path::PathBuf;

use crate::{log::log, read, world};

pub fn convert(input_edition: String, input_version: i32, output_edition: String, output_version: i32, path: PathBuf) {
    log(0,"Reading world...");
    let mut world = match read::read(input_edition, input_version, path.clone()) {
        Some(val) => val,
        None => {
            log(2, format!("Failed to read file at {}",path.clone().to_str().unwrap_or_default()));
            log(0, "Returning...");
            return
        }
    };
    log(0,"Converting world...");
}






