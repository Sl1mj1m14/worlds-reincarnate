//use mc_classic;
//use mc_classic_js;

use std::path::PathBuf;

mod read;
mod write;
//mod convert;

mod functions;
mod world;
mod log;

fn main () {
    println!("hello rewrite");

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

    
}

/*
Blocks
Entities
Items
Player
World Settings





*/



