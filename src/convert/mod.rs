use std::{default, path::PathBuf};

use crate::{log::log, read, world::{self, Value}};

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
    match world.world_data {
        Some(data) => {
            let world_name: String = match data["name"].clone() {
                Value::String(val) => val,
                _ => String::default()
            };
            log(-1,format!("World name is {}",world_name));
        },
        None => log(-1,format!("There is no world data"))
    }
    log(0,"Converting world...");
}






