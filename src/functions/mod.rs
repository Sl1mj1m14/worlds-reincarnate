use std::{fs::File, io::{BufReader,Error}, num::ParseIntError};


pub fn hex2dec (hex: &str) -> Result<i32,ParseIntError> {
    i32::from_str_radix(hex.strip_prefix("0x").unwrap_or(hex), 16)
}

pub fn path2reader (path: &str) -> Result<BufReader<File>,Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader)
}