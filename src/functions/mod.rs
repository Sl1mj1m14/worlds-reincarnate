use std::{fs::File, io::{BufReader,Error}, num::ParseIntError};

// Converts a hexadecimal string into a signed integer
pub fn hex2dec (hex: &str) -> Result<i32,ParseIntError> {
    i32::from_str_radix(hex.strip_prefix("0x").unwrap_or(hex), 16)
}


// Code should be using std::path - not using &str

/*pub fn path2reader (path: &str) -> Result<BufReader<File>,Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader)
}*/