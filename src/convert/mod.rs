use std::collections::HashMap;

use crate::{Handler, convert::worlddata::Data, log::log, version::{FOURK_2, FOURK_EDITION, FOURK_JS, J_C12, J_C13_03, JAVA_EDITION, JAVASCRIPT_EDITION}, world::{Block, BlockArray, Value, World}};

mod read;
mod write;
mod block;
mod worlddata;
mod generate;

#[derive(Clone)]
pub struct Converter {
    worlddata_map: HashMap<Data, Data>,
    block_map: HashMap<Block, Block>
}

pub fn convert(input: Handler, output: Handler) {
    log(0,"Reading world...");
    let mut world = match read::read(input.clone()) {
        Some(val) => val,
        None => {
            log(2, format!("Failed to read file at {}",input.path.clone().to_str().unwrap_or_default()));
            log(0, "Returning...");
            return
        }
    };

    log(0,"Converting world...");

    let world_data = match worlddata::create_map(world.clone().edition, world.clone().version, output.edition.clone(), output.version) {
        Some(m) => m,
        None => {
            log(2, "Failed to build converter - world data");
            return
        }
    };
    
    let block_map = match block::create_map(world.clone().edition, world.clone().version, output.edition.clone(), output.version) {
        Some(m) => m,
        None => {
            log(2, "Failed to build converter - block map");
            return
        }
    };

    let converter: Converter = Converter { 
        worlddata_map: world_data, 
        block_map: block_map
    };

    if world.world_data.is_some() {
        world.world_data = match convert_worlddata(converter.clone(), world.clone(), output.edition.clone(), output.version) {
            Some(w) => Some(w),
            None => {
                log(2, "Failed to convert world data");
                return
            }
        }
    }

    if world.blocks.is_some() {
        world.blocks = match convert_blocks(converter.clone(), world.clone(), output.edition.clone(), output.version) {
            Some(b) => Some(b),
            None => {
                log(2, "Failed to convert blocks");
                return
            }
        }
    }

    world.edition = output.edition;
    world.version = output.version;

    log(0,"Writing world...");
    match write::write(world, output.path, output.args) {
        1 => log(0,"World Converted!"),
        _ => log(2,"Writing failed")
    }
}

fn convert_worlddata (converter: Converter, world: World, _output_edition: String, _output_version: i32) -> Option<HashMap<String,Value>> {
    if world.world_data.is_none() {return None}

    let mut new_world_data: HashMap<String,Value> = HashMap::new();

    for (key, value) in world.world_data.unwrap() {
        let vtype = value.type_as_str().to_string();
        let data = Data {id: key.clone(), ktype: vtype.clone()};
        let new = match converter.worlddata_map.get(&data) {
            Some(v) => v,
            None => {
                log(1, format!("{key} no longer exists, removing"));
                log(-1, format!("type is {}", vtype.clone()));
                continue
            }
        };

        let mut new_value = value.clone();
        if vtype != new.ktype {
            //Handle converting to a new type
            log(1, format!("Unfortunately I do not currently have support to swap from {} to {}",vtype,new.ktype));
            log(1, format!("{key} must be skipped :("));
            continue
        }

        new_world_data.insert(new.id.clone(), new_value);

    }

    return Some(new_world_data);
}

fn convert_blocks (converter: Converter, world: World, output_edition: String, output_version: i32) -> Option<BlockArray> {
    let block_array = match world.blocks {
        Some(b) => b,
        None => {
            log(2, "No blocks, unable to convert!");
            return None
        }
    };
    let mut format = block_array.format.clone();
    let mut dims = block_array.dims.clone();
    let mut default = Block::default();
    
    match output_edition.as_str() {
        JAVA_EDITION => {
            if output_version <= J_C12 {
                format = ["+x".to_string(),"+z".to_string(),"+y".to_string()];
                dims = [256,64,256];
                default = Block { id: Value::UByte(0), block_data: None };
            } else if output_version <= J_C13_03 {
                format = ["+x".to_string(),"+z".to_string(),"+y".to_string()];
                default = Block { id: Value::UByte(0), block_data: None };
            } else {
                log(2, "Unrecognized or unsupported version!");
                return None
            }
        },
        JAVASCRIPT_EDITION => {
            format = ["-x".to_string(),"+z".to_string(),"+y".to_string()];
            let x = dims[0].max(dims[2]);
            dims = [x,64,x];
            default = Block { id: Value::UByte(0), block_data: None };
        },
        FOURK_EDITION => {
            format = ["+z".to_string(),"-y".to_string(),"+x".to_string()];
            dims = [dims[0].min(64), dims[1].min(64), dims[2].min(64)]; //This will be a toggle in the future whether to let the world dimensions get screwed up by a world too small or generate the new blocks here. It's a quirk of the saving mod
            if output_version < FOURK_JS {
                default = Block { id: Value::UInt(0), block_data: None }
            } else {
                default = Block { id: Value::UByte(0), block_data: None }
            }
        }
        _ => {
            log(2, "Unrecognized or unsupported edition!");
            return None
        }
    }

    let mut new_array = block_array.clone();
    let min = [dims[0].min(new_array.dims[0]),dims[1].min(new_array.dims[1]),dims[2].min(new_array.dims[2])];
    if min != new_array.dims { 
        log(0, "Shrinking world");
        new_array = block::shrink_world(new_array.clone(), [min[0]-1,min[1]-1,min[2]-1], [0; 3]); 
    }
    
    log(0, "Converting block ids");
    let mut new_blocks: Vec<Block> = Vec::new();
    for block in new_array.blocks {
        //Handle blockdata first - pull it out and split into what is necessary

        let mut new_block = converter.block_map.get(&block).unwrap_or(&default);

        //Handle blockdata again - merge it back in
        new_blocks.push(new_block.clone());
    }
    new_array.blocks = new_blocks;

    if format != new_array.format {
        log(0, "Rotating world");
        new_array = block::rotate_array(new_array.clone(), format);
    }

    let max = [dims[0].max(new_array.dims[0]),dims[1].max(new_array.dims[1]),dims[2].max(new_array.dims[2])];
    if max != new_array.dims {
        log(0, "Growing world");
        //Eventually set the seed outside using the default for the OUTPUT edition
        let generated: Vec<Block>;
        match output_edition.as_str() {
            JAVA_EDITION => {
                //Add support for generators in the future
                generated = generate::air(output_edition, output_version, max);
            },
            JAVASCRIPT_EDITION => {
                let mut seed: i64 = 1;
                if world.world_data.clone().is_some() && world.world_data.clone().unwrap().get("worldSeed").is_some() {
                    seed = world.world_data.unwrap().get("worldSeed").unwrap().as_i64().unwrap();
                }

                let world_size = max[0].max(max[2]);
                let tiles = generate::javascript(seed, world_size);
                let mut generated1: Vec<Block> = Vec::new();
                for tile in tiles {
                    generated1.push(Block {id: Value::UByte(tile), block_data: None});
                }

                if max[0] != max[2] {
                    let temp_array: BlockArray = BlockArray { 
                        format: new_array.format.clone(), 
                        dims: [world_size, 64, world_size], 
                        blocks: generated1
                    };
                    generated1 = block::shrink_world(temp_array, [max[0]-1,63,max[2]-1], [0; 3]).blocks;
                }

                if max[1] > 64 {
                    let size = (max[1] - 64)*max[0]*max[2];
                    let block = Block {id: Value::UByte(0), block_data: None};
                    generated1.extend_from_slice(&vec![block; size as usize]);
                }

                generated = generated1
            },
            FOURK_EDITION => {
                let mut seed: i64 = 18295169; //This is the hardcoded seed in 4k generation
                if world.world_data.is_some() {
                    let data = world.world_data.unwrap().clone();
                    if data.get("seed").is_some() {
                        seed = data.get("seed").unwrap().as_i64().unwrap();
                    }
                }
                generated = match generate::fourk(output_version, seed, max.clone()) {
                    Some(g) => g,
                    None => {
                        log(2, "Generating missing 4k blocks failed!");
                        return None
                    }
                };
            },
            _ => {
                log(2, "Unrecognized or unsupported edition!");
                return None
            }
        }

        new_array = block::grow_world(new_array.clone(),  max, generated);
    }

    Some(new_array)
}






