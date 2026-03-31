use std::path::PathBuf;

use crate::{log::log, read, world::{self, Value}, write};

mod blocks;

pub fn convert(input_edition: String, input_version: i32, input_path: PathBuf, output_edition: String, output_version: i32, output_path: PathBuf) {
    log(0,"Reading world...");
    let mut world = match read::read(input_edition, input_version, input_path.clone()) {
        Some(val) => val,
        None => {
            log(2, format!("Failed to read file at {}",input_path.clone().to_str().unwrap_or_default()));
            log(0, "Returning...");
            return
        }
    };

    log(0,"Converting world...");
    log(0,"Or rather, pretending to...");
    //Implement converting you fool...

    log(0,"Writing world...");
    match write::write(world, output_path) {
        1 => log(0,"World Converted!"),
        _ => log(2,"Writing failed")
    }
}






