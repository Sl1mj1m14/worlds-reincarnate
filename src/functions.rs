use std::{fs, io::Read, path::PathBuf};
use flate2::read::GzDecoder;

use crate::log::log;

pub enum Dir {
    Home,
    Documents
}

pub fn get_general_dir(dir: Dir) -> PathBuf {
    match dir {
        Dir::Home => {
            let u = directories::UserDirs::new().unwrap();
            u.home_dir().to_path_buf()
        },
        Dir::Documents => {
            let u = directories::UserDirs::new().unwrap();
            u.document_dir().unwrap().to_path_buf()
        }
    }
}

pub fn path2stream (path: PathBuf) -> Option<Vec<u8>> {
    let stream: Vec<u8> = match fs::read(path) {
        Ok(val) => val,
        Err(e) => {
            log(2,format!("{e}"));
            return None
        }
    };

    let mut d_stream = GzDecoder::new(&stream[..]);
    let mut bytes: Vec<u8> = Vec::new();
    d_stream.read_to_end(&mut bytes).unwrap();
    Some(bytes)
}

pub fn stream2string (buf: usize, stream: &[u8], len: u32) -> String {
    let mut chars: Vec<char> = Vec::new();
    for i in 0..len {
        chars.push(stream[buf+(i as usize)] as char);
    }   
    return chars.iter().collect();
}

pub fn stream2short (buf: usize, stream: &[u8]) -> i16 {
    let slice: [u8; 2] = stream[buf..buf+2].try_into().unwrap_or_default();
    i16::from_be_bytes(slice)
}

pub fn stream2ushort (buf: usize, stream: &[u8]) -> u16 {
    let slice: [u8; 2] = stream[buf..buf+2].try_into().unwrap_or_default();
    u16::from_be_bytes(slice)
}

pub fn stream2uint (buf: usize, stream: &[u8]) -> u32 {
    let slice: [u8; 4] = stream[buf..buf+4].try_into().unwrap_or_default();
    u32::from_be_bytes(slice)
}

pub fn stream2long (buf: usize, stream: &[u8]) -> i64 {
    let slice: [u8; 8] = stream[buf..buf+8].try_into().unwrap_or_default();
    i64::from_be_bytes(slice)
}

