//use mc_classic;
//use mc_classic_js;

use std::{fs::{File, create_dir_all, read_dir, remove_dir_all, remove_file}, io::{Error, Read, Write}, mem, path::PathBuf};
use regex::{self, Regex};
use chrono::prelude::*;

mod read;
mod write;
//mod convert;

mod functions;
mod world;
mod log;

const SESSION_DIR: &str = "session";

static mut SESSION: u16 = 0;

fn main () -> Result<(),Error> {
    println!("hello rewrite");

    /***Opening Session File***/

    let mut session: u16 = 0;

    match clean_sessions() {
        Ok(val) => session = val,
        Err(err) => eprintln!("Session Error: {err}")
    }

    unsafe {SESSION = session}
    
    let mut lock_name: String = String::from("session");
    if session != 0 {lock_name += &session.to_string()}
    lock_name += ".lock";

    let session_path: PathBuf = [SESSION_DIR,&lock_name].iter().collect();
    let timestamp: String = Local::now().format("%Y%m%d%H%M%S").to_string();

    let mut session_lock = File::create(session_path.clone())?;
    session_lock.write_all(timestamp.as_bytes())?;
    session_lock = File::open(session_path)?;

    println!("Session Started");


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
    let _ = clean_sessions();
    Ok(())
}

fn clean_sessions () -> Result<u16,Error> {
    let mut session: u16 = 0;
    let mut open: Vec<u16> = Vec::new();

    create_dir_all(SESSION_DIR)?;

    let pattern = match Regex::new(r"^session(?<num>\d*)\.lock$") {
        Ok(val) => val,
        Err(err) => {
            //This error should never happen, but if it does only log files will break
            eprintln!("Regex formatting error: {err}");
            return Ok(0)
        }
    };

    for entry in read_dir(SESSION_DIR)? {
        let path = entry?.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_str().unwrap_or_default();
            if pattern.is_match(file_name) {
                println!("{}",path.display());
                let id: u16 = match pattern.captures(file_name).unwrap()["num"].parse() {
                    Ok(val) => val,
                    Err(_) => 0
                };
                match File::open(path.clone()) {
                    Ok(mut file) => {
                        let mut timestamp: String = String::default();
                        file.read_to_string(&mut timestamp).unwrap_or_default();
                        let _ = log::close(id,timestamp);
                    },
                    Err(_) => {
                        open.push(id);
                        while open.contains(&session) {session += 1}
                        println!("It is read as open...");
                    }
                }
            }
            let _ = remove_file(path);
        } else {
            match remove_dir_all(path) {
                Ok(()) => (),
                Err(err) => {
                    println!("Broken directory inside session folder");
                    eprintln!("Error: {err}");
                }
            }
        }
    }
    Ok(session)
}

/*
Blocks
Entities
Items
Player
World Settings





*/



