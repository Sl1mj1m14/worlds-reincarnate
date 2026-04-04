use std::{collections::HashMap, fs::{File, OpenOptions, create_dir_all, remove_file}, io::Write, path::PathBuf};

use flate2::{Compression, write::GzEncoder};

use crate::{file::Argument, log::log, version::{J_C12, J_C13_03, JAVA_EDITION}, world::{Value, World}};

pub fn write (world: World, path: PathBuf, args: Option<Vec<Argument>>) -> i32 {
    if !path.exists() {
        match create_dir_all(path.clone()) {
            Ok(_) => (),
            Err(e) => {
                log(2, "Failed to find output directory");
                log(2,format!("{e}"));
                return 0
            }
        }
    }

    log(-1,format!("Version is {}",world.version));
    match world.edition.as_str() {
        JAVA_EDITION => {
            if world.version <= J_C12 {
                write_preclassic(world, path.clone())
            } else if world.version <= J_C13_03 {
                write_early_classic(world, path.clone())
            } else {
                log(2, "Unrecognized or unsupported version!");
                0
            }
        },
        _ => {
                log(2, "Unrecognized or unsupported edition!");
                0
        }
    }
}

fn write_preclassic(world: World, dir: PathBuf) -> i32 {
    let mut path: PathBuf = dir;
    path.push("level");
    path.set_extension("dat");

    if path.exists() {
        log(1,"File already exists in output location!");
        log(1,format!("Replacing file at {}",path.clone().to_str().unwrap_or_default()));
        match remove_file(path.clone()) {
            Ok(_) => (),
            Err(e) => {
                log(2,format!("Unable to replace file at {}!",path.clone().to_str().unwrap_or_default()));
                log(2,format!("{e}"));
                return 0
            }
        }
    }

    let mut bytes: Vec<u8> = Vec::new();
    match world.blocks {
        Some(blocks) => {
            for block in blocks.blocks {
                bytes.push( match block.id {
                    Value::UByte(b) => b,
                    _ => {
                        log(1,format!("Invalid block value found! - Replacing with air"));
                        0
                    }
                })
            }
        }
        None => {
            log(2,"No blocks within the world, unable to write");
            return 0
        }
    }

    let output: File = match OpenOptions::new().write(true).create(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            log(2,"Failed to write file!");
            log(2,format!("{e}"));
            return 0
        }
    };

    let mut encoder = GzEncoder::new(output, Compression::default());
    match encoder.write_all(&bytes) {
        Ok(_) => (),
        Err(e) => {
            log(2,"Failed to write file!");
            log(2,format!("{e}"));
            return 0
        }
    }

    return 1
}

fn write_early_classic(world: World, dir: PathBuf) -> i32 {
    let mut path: PathBuf = dir.clone();
    path.push("level");
    path.set_extension("dat");

    log(-1,format!("Original is {}",dir.to_string_lossy()));
    log(-1,format!("Modified is {}",path.to_string_lossy()));
    if path.exists() {
        log(1,"File already exists in output location!");
        log(1,format!("Replacing file at {}",path.clone().to_str().unwrap_or_default()));
        match remove_file(path.clone()) {
            Ok(_) => (),
            Err(e) => {
                log(2,format!("Unable to replace file at {}!",path.clone().to_str().unwrap_or_default()));
                log(2,format!("{e}"));
                return 0
            }
        }
    }

    let mut bytes: Vec<u8> = Vec::new();

    let magic_number: u32 = 0x271BB788;
    bytes.extend_from_slice(&magic_number.to_be_bytes());
    bytes.push(1);

    let world_data: HashMap<String,Value> = world.clone().world_data.unwrap_or_default();

    let mut name = "--".to_string();
    if world_data.clone().contains_key("name") {
        match &world_data["name"] {
            Value::String(s) => name = s.to_string(),
            _ => ()
        }
    }
    let mut len = name.len() as u16;
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(name.as_bytes());

    let mut creator = "unknown".to_string();
    if world_data.clone().contains_key("creator") {
        match &world_data["creator"] {
            Value::String(s) => creator = s.to_string(),
            _ => ()
        }
    }
    len = creator.len() as u16;
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(creator.as_bytes());

    let mut create_time: i64 = 0;
    if world_data.clone().contains_key("createTime") {
        match &world_data["createTime"] {
            Value::Long(l) => create_time = *l,
            _ => ()
        }
    }
    bytes.extend_from_slice(&create_time.to_be_bytes());

    let blocks = match world.blocks {
        Some(b) => b,
        None => {
            log(2, "No blocks, unable to write the world!");
            return 0;
        }
    };

    for dim in blocks.dims {
        bytes.extend_from_slice(&(dim as i16).to_be_bytes());
    }

    for block in blocks.blocks {
        bytes.push(match block.id {
            Value::UByte(b) => b,
            _ => {
                log(1,format!("Invalid block value found! - Replacing with air"));
                0
            }
        })
    }

    let output: File = match OpenOptions::new().write(true).create(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            log(2,"Failed to write file!");
            log(2,format!("{e}"));
            return 0
        }
    };

    let mut encoder = GzEncoder::new(output, Compression::default());
    match encoder.write_all(&bytes) {
        Ok(_) => (),
        Err(e) => {
            log(2,"Failed to write file!");
            log(2,format!("{e}"));
            return 0
        }
    }

    return 1
}