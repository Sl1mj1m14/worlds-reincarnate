use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use jni::{JValue, errors, jni_sig, jni_str, objects::JObject, sys::jbyteArray};
use rusqlite::Connection;
use serde_json::Value as JsonValue;
use snap::raw::Decoder;

use crate::{Handler, convert::generate, file::{Argument, JSFormat, JSUrl}, functions::*, jvm, log::log, version::{self, *}, world::*};

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
            } else if version <= J_C30 {
                read_classic(path)
            } else {
                log(2, "Unrecognized or unsupported version!");
                None
            }
        },
        JAVASCRIPT_EDITION => {
            read_javascript(path, handler.args)
        },
        FOURK_EDITION => {
            read_fourk(path, version)
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
    world_data.insert("version".to_string(), Value::UByte(1));
    buffer += 1;

    let mut len = stream2ushort(buffer, &bytes);
    log(-1,format!("World name length is {len}"));
    buffer += 2;

    let mut string_value = match mutf8::decode(&bytes[buffer..(buffer+(len as usize))]) {
        Ok(val) => val.to_string(),
        Err(e) => {
            log(2, "Unable to parse malformed string from classic world - returning");
            log(2, format!("{e}"));
            return None
        },
    };

    world_data.insert("name".to_string(), Value::String(string_value));
    buffer += len as usize;

    len = stream2ushort(buffer, &bytes);
    buffer += 2;

    string_value = match mutf8::decode(&bytes[buffer..(buffer+(len as usize))]) {
        Ok(val) => val.to_string(),
        Err(e) => {
            log(2, "Unable to parse malformed string from classic world - returning");
            log(2, format!("{e}"));
            return None
        },
    };

    world_data.insert("creator".to_string(), Value::String(string_value));
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

fn read_classic(path: PathBuf) -> Option<World> {

    let Some(stream) = path2stream(path.clone()) else {
        log(1, format!("Failed to read in file at {}!", path.to_string_lossy()));
        return None
    };

    if stream2uint(0, &stream) != 0x271BB788 {
        log(1,"Invalid Classic File - No Magic Number!");
        return None
    }

    if stream[4] != 2 {
        log(1,"Invalid Classic Version!");
        return None
    }

    log(0, "Launching jvm...");

    let Some(jvm) = jvm::launch(jvm::Version::V8, &[r#"-Djava.class.path=target/java"#]) else {
        log(1, "Failed to initialize jvm!");
        return None
    };

    log(0, "JVM has been launched");

    let _ = jvm.attach_current_thread(|env| -> errors::Result<()> {

        let Ok(java_stream) = env.byte_array_from_slice(&stream[5..stream.len()]) else {
            log(1, "Failed to parse byte array, returning");
            return Err(errors::Error::ParseFailed("byte_array".to_string()))
        };

        log(1, "Messages from jvm unable to be logged!");
        log(0, "Messages will only print in terminal");
        println!();

        match env.call_static_method(
            jni_str!("ReadClassic"), 
            jni_str!("read"),
            jni_sig!("([B)V"),
            &[JValue::from(&JObject::from(java_stream))]) {
                Ok(_) => (),
                Err(e) => log(2, format!("{e}")),
            }

        println!();
        log(0, "Resuming standard logging");

        Ok(())
    });




    None
}

fn read_javascript(path: PathBuf, args: Option<Vec<Argument>>) -> Option<World> {
    let mut saved_game_json: JsonValue = JsonValue::Null;
    let mut settings_json: JsonValue = JsonValue::Null;

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

            let val: JsonValue = match serde_json::from_str(str.as_str()) {
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
                    value: row.get(1).unwrap()
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

    log(-1, format!("savedGame: {}",saved_game_json));
    log(-1, format!("settings: {}",settings_json));

    let mut world_data: HashMap<String,Value> = HashMap::new();
    let mut changed_blocks: JsonValue = JsonValue::Null;
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
    let mut tiles = generate::javascript(world_seed, world_size);

    let format = ["-x".to_string(),"+z".to_string(),"+y".to_string()];
    let dims = [world_size, 64, world_size]; //Always in format x/y/z

    log(0, "Inserting Changed Blocks...");
    if changed_blocks.as_object().is_some() {
        for (key, value) in changed_blocks.as_object().unwrap() {
            let coords: Vec<&str> = key.split(['p','_']).filter(|s| !s.is_empty()).collect();
            let x: i32 = coords[0].parse().unwrap_or(-1);
            let y: i32 = coords[1].parse().unwrap_or(-1);
            let z: i32 = coords[2].parse().unwrap_or(-1);
            let bt: i8 = value["bt"].as_i64().unwrap_or(-1) as i8;

            //log(-1, format!("changing block {bt} at {x} {y} {z}"));

            if x < 0 || y < 0 || z < 0 || bt < 0 {continue}

            let i = x + (z * dims[0]) + (y * dims[0] * dims[2]);
            tiles[i as usize] = bt as u8;
        }
    }

    let mut blocks: Vec<Block> = Vec::new();
    for tile in tiles {blocks.push(Block { id: Value::UByte(tile), block_data: None })}

    let block_array = BlockArray {
        format: format,
        dims: dims,
        blocks: blocks
    };

    if settings_json.as_object().is_some() {
        for (key, value) in settings_json.as_object().unwrap() {
            if key == "drawDistance" {
                world_data.insert(key.clone(), Value::UByte(value.as_i64().unwrap() as u8));
                continue
            }
            world_data.insert(key.clone(), Value::json_to_value(value.clone()));
        }
    }

    let mut world = World::default();
    world.world_data = Some(world_data);
    world.blocks = Some(block_array);
    world.edition = JAVASCRIPT_EDITION.to_string();
    world.version = JS_E620;
    log(0,"Assuming latest classic javascript version");

    Some(world)
}

fn read_fourk(path: PathBuf, version: i32) -> Option<World> {
    let seed: i64 = 18295169; //This is the hardcoded seed in 4k generation
    let max_volume = 64*64*64;
    let mut block_array: BlockArray = BlockArray::default();
    block_array.format = ["+z".to_string(),"-y".to_string(),"+x".to_string()];
    block_array.dims = [64; 3]; //Must be in xyz

    let bytes = if version < FOURK_JS {
        let v = match path2stream(path) {
            Some(val) => val,
            None => {
                log(2,format!("Unable to open file"));
                return None
            }
        };
        let mut w: Vec<u8> = Vec::new();
        let mut i: usize = 3;
        while i < v.len() {
            w.push(v[i]);
            i += 4;
        }
        w
    } else {
        match fs::read(path) {
            Ok(val) => val,
            Err(e) => {
                log(2,format!("{e}"));
                return None
            }
        }
    };

    log(-1, format!("Amount of bytes: {}", bytes.len()));
    log(-1, format!("Max Volume: {}", max_volume));
    
    if bytes.len() > max_volume {
        log(1,format!("Assuming this is a 4k world, found {} erroneous bytes",(bytes.len()-max_volume)));
        log(1, "Losing these bytes");
    }

    let mut generated: Vec<Block> = Vec::new();
    if bytes.len() < max_volume {
        log(1,format!("Assuming this is a 4k world, missing {} bytes",(max_volume-bytes.len())));
        log(1, "Generating missing blocks...");
        generated = match generate::fourk(version, seed, block_array.dims.clone()) {
            Some(g) => g,
            None => {
                log(2, "Generating missing 4k blocks failed!");
                return None
            }
        }
    }

    for i in 0..max_volume {
        let block = if i >= bytes.len() {
            generated[i].clone()
        } else if version < FOURK_JS {
            Block { id: Value::UInt(bytes[i] as u32), block_data: None }
        } else {
            Block { id: Value::UByte(bytes[i]), block_data: None }
        };

        block_array.blocks.push(block);
    }

    let mut world = World::default();
    world.edition = FOURK_EDITION.to_string();
    world.version = version;
    world.blocks = Some(block_array.clone());

    if version < FOURK_JS {
        let mut world_data: HashMap<String,Value> = HashMap::new();
        world_data.insert("seed".to_string(), Value::Long(seed));
        world.world_data = Some(world_data);
    }

    //log(-1, format!("{:?}",block_array));

    Some(world)
}