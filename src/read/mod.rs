use std::{fs, io::Read, path::PathBuf};
use flate2::read::GzDecoder;

use crate::{log::log, version::{J_C12, JAVA_EDITION}, world::{Block, BlockArray, Value, World}};

pub fn read (edition: String, version: i32, path: PathBuf) -> Option<World> {
    match edition.as_str() {
        JAVA_EDITION => {
            if version <= J_C12 {
                read_preclassic(path)
            } else {
                None
            }
        },
        _ => None
    }
}

fn read_preclassic (path: PathBuf) -> Option<World> {
    let max_volume = 256*256*64;
    let stream: Vec<u8> = match fs::read(path) {
        Ok(val) => val,
        Err(e) => {
            log(2,format!("Unable to open file"));
            log(2,format!("{e}"));
            return None
        }
    };

    let mut d_stream = GzDecoder::new(&stream[..]);
    let mut bytes: Vec<u8> = Vec::new();
    d_stream.read_to_end(&mut bytes).unwrap();

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