use std::path::PathBuf;

use crate::{Handler, log::log, world::{self, Value}};

mod read;
mod write;
mod block;
pub(crate) mod generate;

pub fn convert(input: Handler, output: Handler) {
    /*log(0,"Reading world...");
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
    }*/
}






