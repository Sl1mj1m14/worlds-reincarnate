//use crate::SESSION

const LOG_DIR: &str = "logs";

use std::{io::Error, path::{Path, PathBuf}};

pub fn log (msg: String) {
    //Handle errors
    //Write to log file
    println!("{msg}");
}

pub fn close (session: u16, timestamp: String) -> Result<(),Error> {

    let mut log_name: String = String::from("log");
    if session != 0 {log_name += &session.to_string()}
    log_name += ".txt";
    let path: PathBuf = [LOG_DIR,&log_name].iter().collect();
    println!("Original Path is: {}",path.display());
    if !path.exists() {return Ok(())}
    
    let mut append: i32 = 0;

    let mut rename: String = String::from("log_");
    rename += &timestamp;
    let mut rename_path: PathBuf = [LOG_DIR,&rename,".txt"].iter().collect();

    Ok(())

}

