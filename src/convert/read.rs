use std::{collections::HashMap, path::PathBuf};

use crate::{Handler, functions::*, log::log, version::*, world::*};

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
    block_array.dims = [256,256,64];
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