use std::{collections::HashMap, default, fs, io::Read, path::PathBuf};

use rusqlite::Connection;
use serde_json::Value as JValue;
use snap::{raw::Decoder, read::FrameDecoder};

use crate::{Handler, convert::generate, file::{Argument, JSFormat, JSUrl}, functions::*, log::log, version::*, world::*};

pub fn read (handler: Handler) -> Option<World> {
    let edition = handler.edition.clone();
    let version = handler.version;
    let path = handler.path.clone();

    match edition.as_str() {
        JAVA_EDITION => {
            if version <= J_C12 {
                read_preclassic(path)
            } else if version <= J_C13_03 {
                read_early_classic(path)
            } else {
                log(2, "Unrecognized or unsupported version!");
                None
            }
        },
        JAVASCRIPT_EDITION => {
            read_javascript(path, handler.args)
        },
        _ => {
                log(2, "Unrecognized or unsupported edition!");
                None
        }
    }
}

fn read_preclassic (path: PathBuf) -> Option<World> {
    let max_volume = 256*256*64;
    let bytes = match path2stream(path) {
        Some(val) => val,
        None => {
            log(2,format!("Unable to open file"));
            return None
        }
    };

    let mut block_array: BlockArray = BlockArray::default();
    block_array.format = ["+x".to_string(),"+z".to_string(),"+y".to_string()];
    block_array.dims = [256,64,256]; //Must be in xyz
    if bytes.len() > max_volume {
        log(1,format!("Assuming this is a preclassic world, found {} erroneous bytes",(bytes.len()-max_volume)));
        log(1, "Losing these bytes");
    }
    for i in 0..max_volume {
        block_array.blocks.push(Block { id: Value::new(bytes[i]), block_data: None })
    }

    let mut world = World::default();
    world.edition = JAVA_EDITION.to_string();
    world.version = J_C12;
    log(0,"Assuming latest pre classic version");
    world.blocks = Some(block_array);

    Some(world)
}

fn read_early_classic (path: PathBuf) -> Option<World> {
    let bytes = match path2stream(path) {
        Some(val) => val,
        None => {
            log(2,format!("Unable to open file"));
            return None
        }
    };
    let mut buffer: usize = 0;

    if stream2uint(buffer, &bytes) != 0x271BB788 {
        log(2,"Invalid Classic File - No Magic Number!");
        return None
    }
    buffer += 4;

    let mut world_data: HashMap<String,Value> = HashMap::new();

    if bytes[buffer] != 1 {
        log(2,"Invalid Classic Version!");
        //Attempt to open with regular classic
        return None
    }
    world_data.insert("version".to_string(), Value::Byte(1));
    buffer += 1;

    let mut len = stream2ushort(buffer, &bytes);
    log(-1,format!("World name length is {len}"));
    buffer += 2;
    world_data.insert("name".to_string(), Value::String(stream2string(buffer, &bytes, len as u32)));
    buffer += len as usize;

    len = stream2ushort(buffer, &bytes);
    buffer += 2;
    world_data.insert("creator".to_string(), Value::String(stream2string(buffer, &bytes, len as u32)));
    buffer += len as usize;

    world_data.insert("createTime".to_string(), Value::Long(stream2long(buffer, &bytes)));
    buffer += 8;

    let mut block_array = BlockArray::default();
    block_array.format = ["+x".to_string(),"+z".to_string(),"+y".to_string()];

    for i in 0..3 {
        block_array.dims[i as usize] = stream2short(buffer, &bytes) as i32;
        buffer += 2;
    }

    block_array.dims.swap(1, 2); //Dims must be in xyz format

    for _ in 0..(block_array.dims[0]*block_array.dims[1]*block_array.dims[2]) {
        block_array.blocks.push(Block { id: Value::new(bytes[buffer]), block_data: None });
        buffer += 1;
    }

    if bytes.len() > buffer {
        log(1,format!("Extra bytes remaining in file, could be corrupted!"));
        log(1,format!("{} bytes skipped!",bytes.len()-buffer));
    }

    let mut world: World = World::default();
    world.world_data = Some(world_data);
    world.blocks = Some(block_array);
    world.edition = JAVA_EDITION.to_string();
    world.version = J_C13_03;
    log(0,"Assuming latest early classic version");

    Some(world)
}

fn read_javascript(path: PathBuf, args: Option<Vec<Argument>>) -> Option<World> {
    let mut saved_game_json: JValue = JValue::Null;
    let mut settings_json: JValue = JValue::Null;

    let mut format = JSFormat::Raw;
    let mut _url = JSUrl::Classic;

    if args.is_some() {
        for arg in args.unwrap() {
            match arg {
                Argument::JSFormat(f) => format = f,
                Argument::JSUrl(u) => _url = u,
                _ => ()
            }
        }
    }

    match format {
        JSFormat::Raw => {

            let str: String = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    log(1, "Unable to read world file");
                    log(2,format!("{e}"));
                    return None
                }
            };

            let val: JValue = match serde_json::from_str(str.as_str()) {
                Ok(v) => v,
                Err(e) => {
                    log(1, "Unable to parse json");
                    log(2,format!("{e}"));
                    return None
                }
            };

            if val.get("savedGame").is_some() {
                saved_game_json = val["savedGame"].clone();
            }

            if val.get("settings").is_some() {
                settings_json = val["settings"].clone();
            }
        },
        JSFormat::Firefox => {
            struct Data {
                key: String,
                value: Vec<u8>
            }

            let mut path = path;
            path.push("ls");
            path.push("data");
            path.set_extension("sqlite");

            let conn: Connection = match Connection::open(path) {
                Ok(c) => c,
                Err(e) => {
                    log(1, "Unable to open Javascript world");
                    log(2,format!("{e}"));
                    return None
                }
            };

            let mut stmt = match conn.prepare("SELECT key, value FROM data;") {
                Ok(s) => s,
                Err(e) => {
                    log(1, "SQL Parsing Error");
                    log(2,format!("{e}"));
                    return None
                }
            };

            let table = match stmt.query_map([], |row| Ok(
                Data {
                    key: row.get(0).unwrap(),
                    value: row.get(2).unwrap()
                }
            )) {
                Ok(t) => t,
                Err(e) => {
                    log(1, "SQL Parsing Error");
                    log(2,format!("{e}"));
                    return None
                }
            };

            for data in table {
                if data.is_err() {continue}

                let data = data.unwrap();
                log(-1, format!("Key is {}",data.key));

                let decompressed = match Decoder::decompress_vec(&mut Decoder::new(), &data.value) {
                    Ok (v) => v,
                    Err(e) => {
                        log(2,format!("{e}"));
                        Vec::new()
                    }
                };

                let str = String::from_utf8(decompressed).unwrap_or(String::default());
                log(-1, format!("{}",str.clone()));

                if data.key == "savedGame" {
                    saved_game_json = match serde_json::from_str(&str.as_str()) {
                        Ok(w) => w,
                        Err(e) => {
                            log(2, "Invalid JSON String!");
                            log(2,format!("{e}"));
                            return None
                        }
                    };
                }

                if data.key == "settings" {
                    settings_json = match serde_json::from_str(&str.as_str()) {
                        Ok(w) => w,
                        Err(_) => {
                            log(1,"Malformed settings string - skipping");
                            settings_json
                        }
                    };
                }
            }
        },
        _ => {
            log(2, "Attempting to read an unsupported Javascript Format - support should be coming soon!:tm:");
            return None
        }
    }

    log(-1, format!("{}",saved_game_json));
    log(-1, format!("{}",settings_json));

    let mut world_data: HashMap<String,Value> = HashMap::new();
    let mut changed_blocks: JValue = JValue::Null;
    let mut world_size: i32 = 0;
    let mut world_seed: i64 = 0;

    match saved_game_json.as_object() {
        Some(obj) => {
            for (key, value) in obj {
                if key == "changedBlocks" { 
                    changed_blocks = value.clone();
                    continue
                }
                if key == "worldSize" {
                    world_size = value.as_u64().unwrap() as i32;
                    continue
                }
                if key == "version" {
                    world_data.insert(key.clone(), Value::UByte(value.as_i64().unwrap() as u8));
                    continue
                }
                if key == "worldSeed" {
                    world_data.insert(key.clone(), Value::Long(value.as_i64().unwrap()));
                    world_seed = value.as_i64().unwrap();
                    continue
                }
                world_data.insert(key.clone(), Value::json_to_value(value.clone()));
            }
        },
        None => {
           log(2, "Failed to retreive a Javascript Saved Game!");
           return None 
        }
    }

    if world_seed == 0 || world_size == 0 {
        log(2, "Insufficient information to build Javascript world, returning");
        return None
    }

    log(0, "Generating Javascript World from Seed...");
    //let mut tiles = generate::javascript(world_seed, world_size);

    if settings_json.as_object().is_some() {
        for (key, value) in settings_json.as_object().unwrap() {
            world_data.insert(key.clone(), Value::json_to_value(value.clone()));
        }
    }

    
    None
}