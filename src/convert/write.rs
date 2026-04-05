use std::{collections::HashMap, default, fs::{File, OpenOptions, create_dir_all, remove_file}, io::Write, path::PathBuf};

use serde_json::{Map, Value as JValue, json};
use flate2::{Compression, write::GzEncoder};

use crate::{convert::generate, file::Argument, log::log, version::{J_C12, J_C13_03, JAVA_EDITION, JAVASCRIPT_EDITION}, world::{self, Value, World}};

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
        JAVASCRIPT_EDITION => write_javascript(world, path.clone(), args),
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

    //Dims are always stored in xyz format
    //Classic uses xzy
    bytes.extend_from_slice(&(blocks.dims[0] as i16).to_be_bytes());
    bytes.extend_from_slice(&(blocks.dims[2] as i16).to_be_bytes());
    bytes.extend_from_slice(&(blocks.dims[1] as i16).to_be_bytes());

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

fn write_javascript (world: World, dir: PathBuf, args: Option<Vec<Argument>>) -> i32 {
    if world.blocks.is_none() {
        log(2, "Insufficient information to write world - no world size or blocks!");
        return 0
    }

    let block_array = world.blocks.unwrap();

    let world_data: HashMap<String, Value> = match world.world_data {
        Some(w) => w,
        None => HashMap::default()
    };

    let mut world_size = block_array.dims[0];
    let mut world_seed: i64 = 1; //Default seed cannot be 0 in classic javascript

    if world_data.contains_key("worldSeed") {
        world_seed = match world_data["worldSeed"] {
            Value::Long(l) => l,
            Value::ULong(u) => u as i64,
            Value::Double(d) => d as i64,
            _ => world_seed
        }
    }

    if (world_size * world_size * 64) as usize != block_array.blocks.len() {
        log(1, format!("World size of {} does not match how many blocks there are?", world_size));
        log(1, format!("Attempting to repair size"));
        world_size = ((block_array.blocks.len()/64) as f32).sqrt() as i32;
        if (world_size * world_size * 64) as usize != block_array.blocks.len() {
            log(2, format!("Failed to repair invalid world size"));
            return 1
        }
    }

    let mut block_map = Map::new();
    let tiles = generate::javascript(world_seed, world_size);

    log(0, "Optimizing file size...");
    
    for x in 0..world_size {
        for y in 0..64 {
            for z in 0..world_size {
                let i = (x + (z * world_size) + (y * world_size * world_size)) as usize;
                let block = match block_array.blocks[i].id.as_u8() {
                    Some(b) => b,
                    None => {
                        log(1, "Invalid block found - skipping");
                        continue
                    }
                };

                if tiles[i] != block {
                    let key: String = format!("p{x}_{y}_{z}");
                    let value: JValue = json!({"a": 1, "bt": block});
                    block_map.insert(key, value);
                }
            }
        }
    }

    log(0, "Building JSON...");

    let saved_game = json!({
        "changedBlocks" : block_map,
        "version" : 1,
        "worldSeed" : world_seed,
        "worldSize" : world_size
    });

    let settings = json!({
        "backward" : world_data.get("backward").unwrap_or(&Value::String("S".to_string())).as_string().unwrap(),
        "build" : world_data.get("build").unwrap_or(&Value::String("B".to_string())).as_string().unwrap(),
        "chat" : world_data.get("chat").unwrap_or(&Value::String("T".to_string())).as_string().unwrap(),
        "drawDistance" : world_data.get("drawDistance").unwrap_or(&Value::Byte(0)).as_i8().unwrap(),
        "fog" : world_data.get("fog").unwrap_or(&Value::String("F".to_string())).as_string().unwrap(),
        "forward" : world_data.get("forward").unwrap_or(&Value::String("W".to_string())).as_string().unwrap(),
        "fps" : world_data.get("fps").unwrap_or(&Value::Boolean(false)).as_bool().unwrap(),
        "invert" : world_data.get("invert").unwrap_or(&Value::Boolean(false)).as_bool().unwrap(),
        "jump" : world_data.get("jump").unwrap_or(&Value::String("<space>".to_string())).as_string().unwrap(),
        "left" : world_data.get("left").unwrap_or(&Value::String("A".to_string())).as_string().unwrap(),
        "loadLoc" : world_data.get("loadLoc").unwrap_or(&Value::String("R".to_string())).as_string().unwrap(),
        "music" : world_data.get("music").unwrap_or(&Value::Boolean(true)).as_bool().unwrap(),
        "right" : world_data.get("right").unwrap_or(&Value::String("D".to_string())).as_string().unwrap(),
        "saveLoc" : world_data.get("saveLoc").unwrap_or(&Value::String("<enter>".to_string())).as_string().unwrap(),
        "sound" : world_data.get("sound").unwrap_or(&Value::Boolean(true)).as_bool().unwrap(),
        "username" : world_data.get("username").unwrap_or(&Value::String("".to_string())).as_string().unwrap()
    });

    log(-1, format!("savedGame: {}", saved_game.to_string()));
    log(-1, format!("settings: {}", settings.to_string()));

    return 0
}