use std::{fs::{File, OpenOptions, create_dir_all, remove_file}, io::Write, path::PathBuf};

use flate2::{Compression, write::GzEncoder};

use crate::{log::log, version::{J_C12, J_C13, JAVA_EDITION}, world::{Value, World}};

const OUTPUT_DIR: &str = "output";

pub fn write (world: World) -> i32 {
    let dir: PathBuf = [OUTPUT_DIR].iter().collect();
    if !dir.exists() {
        match create_dir_all(dir) {
            Ok(_) => (),
            Err(e) => {
                log(2, "Failed to create output directory");
                log(2,format!("{e}"));
                return 0
            }
        }
    }
    
    match world.edition.as_str() {
        JAVA_EDITION => {
            if world.version <= J_C12 {
                write_preclassic(world)
            } else if world.version <= J_C13 {
                write_early_classic(world)
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

fn write_preclassic(world: World) -> i32 {
    let path: PathBuf = [OUTPUT_DIR,"level.dat"].iter().collect();

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

fn write_early_classic(world: World) -> i32 {
    1
}