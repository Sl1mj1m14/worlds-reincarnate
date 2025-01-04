use std::{fs::read, path::PathBuf};
use mc_classic;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Button Failed")]
    Generic()
}

pub fn check_file_type (path: PathBuf) -> Result<String,ReadError> {
    let ext: Vec<&str> = path.clone().to_str().unwrap_or("").split("/").collect();
    //read(path)
    return Ok("Unknown".to_string())
}