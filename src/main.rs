use std::sync::OnceLock;

use chrono::prelude::*;

//mod read;
mod write;
//mod convert;

mod functions;
mod world;
mod log;

static TIMESTAMP: OnceLock<String> = OnceLock::new();
const DEFAULT_TIMESTAMP: &str = "19700101120000";

fn main () {

    let timestamp: String = Local::now().format("%Y%m%d%H%M%S").to_string();
    TIMESTAMP.set(timestamp).unwrap();

    log::start();
    log::log(0,format!("Session Started at {}",Local::now().format("%Y-%m-%d %H:%M:%S")));

/*
    let level = read::read_classic_file(PathBuf::from("input/level.dat"));

    match level {
        Ok(value) => {
            println!("I guess this worked?");
            let mut name = String::from("");
            if value.world_data.is_some() {
                //if value.world_data.unwrap().
            }
            println!("World Name is: ")
        },
        Err(error) => eprintln!("{error}")
    }

    */
    //mem::drop(session_lock);
}



/*
Blocks
Entities
Items
Player
World Settings





*/



