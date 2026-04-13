use std::{collections::HashMap, fs::{self, File, OpenOptions, create_dir_all, remove_dir_all, remove_file}, io::{BufWriter, Write}, path::PathBuf};

use chrono::Utc;
use regex::Regex;
use rusqlite::Connection;
use serde_json::{Map, Value as JValue, json, to_writer_pretty};
use flate2::{Compression, write::GzEncoder};
use snap::raw::Encoder;

use crate::{convert::generate, file::{Argument, JSFormat, JSUrl}, log::log, version::{J_C12, J_C13_03, JAVA_EDITION, JAVASCRIPT_EDITION}, world::{Value, World}};

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

    let mut mutf8 = mutf8::encode(name.as_str());
    let mut len = mutf8.len() as u16;
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(&mutf8);

    let mut creator = "unknown".to_string();
    if world_data.clone().contains_key("creator") {
        match &world_data["creator"] {
            Value::String(s) => creator = s.to_string(),
            _ => ()
        }
    }

    mutf8 = mutf8::encode(creator.as_str());
    len = mutf8.len() as u16;
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(&mutf8);

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
                log(1,format!("Invalid block value {:?} found! - Replacing with air", block));
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
            return 0
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
        "music" : world_data.get("music").unwrap_or(&Value::Boolean(false)).as_bool().unwrap(),
        "right" : world_data.get("right").unwrap_or(&Value::String("D".to_string())).as_string().unwrap(),
        "saveLoc" : world_data.get("saveLoc").unwrap_or(&Value::String("<enter>".to_string())).as_string().unwrap(),
        "sound" : world_data.get("sound").unwrap_or(&Value::Boolean(true)).as_bool().unwrap(),
        "username" : world_data.get("username").unwrap_or(&Value::String("".to_string())).as_string().unwrap()
    });

    log(-1, format!("savedGame: {}", saved_game.to_string()));
    log(-1, format!("settings: {}", settings.to_string()));

    let mut format = JSFormat::Raw;
    let mut url = JSUrl::Classic;

    if args.is_some() {
        for arg in args.unwrap() {
            match arg {
                Argument::JSFormat(f) => format = f,
                Argument::JSUrl(u) => url = u,
                _ => ()
            }
        }
    }

    match format {
        JSFormat::Raw => {
            let value: JValue = json!({ "savedGame" : saved_game, "settings" : settings});

            let mut path: PathBuf = dir.clone();
            path.push("output");
            path.set_extension("json");

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

            let output: File = match OpenOptions::new().write(true).create(true).open(path) {
                Ok(f) => f,
                Err(e) => {
                    log(2,"Failed to create file!");
                    log(2,format!("{e}"));
                    return 0
                }
            };
            let writer = BufWriter::new(output);

            match to_writer_pretty(writer, &value) {
                Ok(_) => (),
                Err(e) => {
                    log(2,"Failed to write file!");
                    log(2,format!("{e}"));
                    return 0
                }
            }
            
        },
        JSFormat::Firefox => {

            //A lot of information for this section is courtesy of this: https://www.cclsolutionsgroup.com/post/local-storage-and-session-storage-in-mozilla-firefox-part-1

            let domain: String = match url {
                JSUrl::Classic => "https://classic.minecraft.net".to_string(),
                JSUrl::Omniarchive => "https://omniarchive.uk".to_string(),
                JSUrl::LocalHost(port) => format!("http://localhost:{port}"), //Test parsing with good laptop
                JSUrl::Other(site) => {
                    let pattern = Regex::new(r#"^[a-zA-Z0-9\.\+-]+://[^/\#\\]+"#).unwrap(); //Pattern to match "scheme://hostname" - removing any sub folders
                    let captures = pattern.captures(&site);
                    match captures {
                        Some(c) => c[0].to_string(),
                        None => {
                            log(1, format!("Invalid url {site} detected - will likely not work in a browser"));
                            site
                        }
                    }
                }
            };

            let mut pattern = Regex::new(r#"\?"#).unwrap();
            log(-1, "Regex 1 is fine");
            let mut folder = pattern.replace_all(&domain, "^").to_string();
            pattern = Regex::new(r#"[:\*\"\\/\|<>]"#).unwrap(); //Pattern based on mozilla docs here: https://hg-edge.mozilla.org/mozilla-central/file/tip/dom/quota/ActorsParent.cpp
            log(-1, "Regex 2 is fine");
            folder = pattern.replace_all(&folder, "+").to_string();

            log(-1, format!("Folder name is {}",folder.clone()));
            let mut path = dir.clone();
            path.push(folder);

            if path.exists() {
                log(1,"Folder already exists in output location!");
                log(1,format!("Replacing folder at {}",path.clone().to_str().unwrap_or_default()));
                match remove_dir_all(path.clone()) {
                    Ok(_) => (),
                    Err(e) => {
                        log(2,format!("Unable to replace file at {}!",path.clone().to_str().unwrap_or_default()));
                        log(2,format!("{e}"));
                        return 0
                    }
                }
            }

            path.push("ls");
            match create_dir_all(path.clone()) {
                Ok(_) => (),
                Err(e) => {
                    log(2, "Failed to create folder");
                    log(2,format!("{e}"));
                    return 0
                }
            }

            path.push("data");
            path.set_extension("sqlite");

            let conn = match Connection::open(path.clone()) {
                Ok(c) => c,
                Err(e) => {
                    log(2, "Failed to write to database");
                    log(2,format!("{e}"));
                    return 0
                }
            };

            let _ = conn.pragma_update(None, "user_version", 80);
            let _ = conn.pragma_update(None, "auto_vacuum", 2);
            let _ = conn.pragma_update(None, "page_size", 1024);
            match conn.execute("VACUUM", []) {
                Ok(_) => (),
                Err(e) => {
                    log(2, "Failed to write to database");
                    log(2,format!("{e}"));
                    return 0
                }
            }

            if conn.execute(
            "CREATE TABLE data ( 
                key TEXT PRIMARY KEY, 
                utf16_length INTEGER NOT NULL, 
                conversion_type INTEGER NOT NULL, 
                compression_type INTEGER NOT NULL, 
                last_access_time INTEGER NOT NULL DEFAULT 0, 
                value BLOB NOT NULL)", 
        []
            ).is_err() {
                log(2, "Failed to write to database");
                return 0
            }

            if conn.execute(
            "CREATE TABLE if not exists database ( 
                origin TEXT NOT NULL, 
                usage INTEGER NOT NULL DEFAULT 0, 
                last_vacuum_time INTEGER NOT NULL DEFAULT 0, 
                last_analyze_time INTEGER NOT NULL DEFAULT 0, 
                last_vacuum_size INTEGER NOT NULL DEFAULT 0)",
        []
            ).is_err() {
                log(2, "Failed to write to database");
                return 0
            }

            let mut map: HashMap<String,JValue> = HashMap::new();
            map.insert("savedGame".to_string(), saved_game);
            map.insert("settings".to_string(), settings);

            let mut usage: i32 = 0;
            for (key, value) in map {
                let json = value.to_string();

                let utf16_len = json.encode_utf16().count() as i32;
                usage += key.encode_utf16().count() as i32;
                usage += utf16_len;

                let compressed = match Encoder::compress_vec(&mut Encoder::new(), json.as_bytes()) {
                    Ok(b) => b,
                    Err(e) => {
                        log(2, "Failed to compress bytes");
                        log(2,format!("{e}"));
                        return 0
                    }
                };

                match conn.execute("INSERT into data VALUES (?1, ?2, ?3, ?4, ?5, ?6)", (key.clone(), utf16_len, 1, 1, 0, compressed)) {
                    Ok(_) => (),
                    Err(e) => {
                        log(2, format!("Failed to insert {}", key));
                        log(2,format!("{e}"));
                        return 0
                    }
                }
            }

            let timestamp = Utc::now().timestamp_micros();
            let vacuum = match fs::metadata(path.clone()) {
                Ok(v) => v.len() as i64,
                Err(e) => {
                    log(2, "Unable to read vacuum size");
                    log(2,format!("{e}"));
                    return 0
                }
            };

            match conn.execute("INSERT into database VALUES (?1, ?2, ?3, ?4, ?5)", (domain.clone(), usage, timestamp, 0, vacuum)) {
                Ok(_) => (),
                Err(e) => {
                    log(2, "Failed to insert into main database");
                    log(2,format!("{e}"));
                    return 0
                }
            }

            for _ in 0..2 {path.pop();}
            path.push(".metadata-v2");

            //Building metadata file
            let mut metadata: Vec<u8> = Vec::new();
            metadata.extend_from_slice(&timestamp.to_be_bytes()); //Timestamp
            metadata.push(0); //Persisted
            metadata.extend_from_slice(&(0 as i32).to_be_bytes()); //Suffix
            metadata.extend_from_slice(&(0 as i32).to_be_bytes()); //Group
            metadata.extend_from_slice(&(domain.len() as i32).to_be_bytes()); //Length of origin string (domain)
            metadata.extend_from_slice(domain.as_bytes()); //Origin (domain)
            metadata.push(0); //Is App

            match fs::write(path.clone(), metadata) {
                Ok(_) => (),
                Err(e) => {
                    log(1, "Failed to write metadata file - skipping");
                    log(2,format!("{e}"));
                }
            }
            return 1
        },
        _ => {
            log(2, "Attempting to write an unsupported Javascript Format - support should be coming soon!:tm:");
            return 0
        }
    }

    return 1
}