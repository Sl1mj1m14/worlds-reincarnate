use std::{collections::HashMap, path::PathBuf};

use csv::StringRecord;

use crate::{log::log, resources::{self, Map, Resource}, version::{FOURK_EDITION, FOURK_JS, JAVA_EDITION, JAVASCRIPT_EDITION}, world::{Block, BlockArray, Value}};

pub fn create_map(input_edition: String, input_version: i32, output_edition: String, output_version: i32) -> Option<HashMap<Block, Block>> {
    
    let mut path: PathBuf = resources::HASHES.get().unwrap()[&Resource::Map(Map::Block)].path.clone();

    if !path.exists() {
        log(2, "Block map not found, unable to convert world!");
        return None
    }

    let mut reader = match csv::ReaderBuilder::new().has_headers(false).from_path(path) {
        Ok(r) => r,
        Err(e) => {
            log(2, "Failed to parse block map, unable to convert world!");
            log(2, format!("{e}"));
            return None
        }
    };

    let mut map: HashMap<Block, Block> = HashMap::new();

    let input_type = get_block_type(input_edition.clone(), input_version);
    let output_type = get_block_type(output_edition.clone(), output_version);

    let mut input_indices: Vec<usize> = Vec::new();
    let mut output_indices: Vec<usize> = Vec::new();

    for (index, result) in reader.records().enumerate() {
        let line = match result {
            Ok(r) => r,
            Err(_) => {
                log(1, "Invalid record when building block map - skipping");
                continue
            }
        };

        if index == 0 {
            let mut i = 1; //Start index at 1, Column 0 is used for readable format
            let mut current = String::default();
            while i < line.iter().count() {
                log(-1, line.get(i).unwrap());
                if line.get(i).unwrap() != String::default() { current = line.get(i).unwrap().to_string(); }
                if current == input_edition.clone() { input_indices.push(i) }
                if current == output_edition.clone() { output_indices.push(i) }
                i += 4; //Each grouping accounts for 4 entries - start version, end version, id, blockdata
            }
            continue
        }

        let key = match record_to_block(input_indices.clone(), line.clone(), input_type.clone(), input_version) {
            Some(b) => b,
            None => continue
        };

        let value = match record_to_block(output_indices.clone(), line, output_type.clone(), output_version) {
            Some(b) => b,
            None => continue
        };

        map.insert(key, value);

    }

    for (key, value) in map.clone() {
        log(-1, format!("Key: {:?}", key));
        log(-1, format!("Value: {:?}", value));
    }

    //Overriding entries with special cases
    path = resources::HASHES.get().unwrap()[&Resource::Map(Map::BlockSpecial)].path.clone();

    if !path.exists() {
        log(1, "Special block map not found, potential for incomplete block conversion");
        return Some(map)
    }

    reader = match csv::ReaderBuilder::new().has_headers(false).from_path(path) {
        Ok(r) => r,
        Err(e) => {
            log(1, "Failed to parse special block map, potential for incomplete block conversion");
            log(1, format!("{e}"));
            return Some(map)
        }
    };

    for (_, result) in reader.records().enumerate() {
        let line = match result {
            Ok(r) => r,
            Err(_) => {
                log(1, "Invalid record when building block map - skipping");
                continue
            }
        };

        let edition: &str = line.get(0).unwrap();
        let lower: i32 = line.get(1).unwrap().parse().unwrap_or(input_version + 1);
        let upper: i32 = line.get(2).unwrap().parse().unwrap_or(0);

        if input_edition != edition || input_version < lower || input_version > upper {continue}

        let raw_1: &str = line.get(3).unwrap_or_default();
        let raw_2: &str = line.get(4).unwrap_or_default();

        let key = match raw_to_block([raw_1, raw_2], input_type.clone()) {
            Some(b) => b,
            None => continue,
        };

        let mut i: usize = 5;
        loop {
            let edition = match line.get(i) {
                Some(s) => {
                    log(-1, format!("edition is: {s}"));
                    s
                },
                None => break
            };
            let lower: i32 = line.get(i+1).unwrap().parse().unwrap_or(output_version + 1);
            let upper: i32 = line.get(i+2).unwrap().parse().unwrap_or(0);

            if output_edition != edition || output_version < lower || output_version > upper {
                i += 5;
                continue
            }

            let raw_1: &str = line.get(i+3).unwrap_or_default();
            let raw_2: &str = line.get(i+4).unwrap_or_default();

            match raw_to_block([raw_1, raw_2], output_type.clone()) {
                Some(b) => {
                    map.insert(key, b);
                    break
                },
                None => {
                    i += 5;
                    continue
                }
            }
        }
    }

    Some(map)
}

pub fn rotate_array (input_array: BlockArray, output_format: [String; 3]) -> BlockArray {
    let mut output_blocks: Vec<Block> = vec![Block::default(); input_array.blocks.len()];

    let in_mults: Vec<i32> = format_to_mults(input_array.format, input_array.dims);
    let out_mults: Vec<i32> = format_to_mults(output_format.clone(), input_array.dims);

    for x in 0..input_array.dims[0] {
        for y in 0..input_array.dims[1] {
            for z in 0..input_array.dims[2] {
                let x1 = (out_mults[0] - x).abs()*out_mults[1];
                let y1 = (out_mults[2] - y).abs()*out_mults[3];
                let z1 = (out_mults[4] - z).abs()*out_mults[5];

                let x2 = (in_mults[0] - x).abs()*in_mults[1];
                let y2 = (in_mults[2] - y).abs()*in_mults[3];
                let z2 = (in_mults[4] - z).abs()*in_mults[5];

                output_blocks[(x1+y1+z1) as usize] = input_array.blocks[(x2+y2+z2) as usize].clone();
            }
        }
    }

    BlockArray {
        format: output_format,
        dims: input_array.dims,
        blocks: output_blocks
    }
}

//The bounds should be a range to define the size of the world, the dimension arrays are in 'xyz' format
//For example, if the world is going to be 64 blocks from the bottom, the bounds should be 0 and 63
pub fn shrink_world (world: BlockArray, upper_bounds: [i32; 3], lower_bounds: [i32; 3]) -> BlockArray {

    //Shrinking is probably broken!
    let mults: Vec<i32> = format_to_mults(world.format.clone(), world.dims);

    let mut output_dims: [i32; 3] = [0; 3];
    for i in 0..3 {
        output_dims[i] = upper_bounds[i]-lower_bounds[i]+1
    }

    let volume = output_dims[0]*output_dims[1]*output_dims[2];
    let mut output_blocks: Vec<Block> = vec![Block::default(); volume as usize];

    for x in 0..world.dims[0] {
        for y in 0..world.dims[1] {
            for z in 0..world.dims[2] {

                //Flipping coordinates prior to checking if within bounds
                let mut x1 = (mults[0] - x).abs();
                let mut y1 = (mults[2] - y).abs();
                let mut z1 = (mults[4] - z).abs();
                
                //Only pushing blocks if they are within the new world size
                if
                    x1 >= lower_bounds[0] && x1 <= upper_bounds[0] &&
                    y1 >= lower_bounds[1] && y1 <= upper_bounds[1] &&
                    z1 >= lower_bounds[2] && z1 <= upper_bounds[2]
                { 
                    
                    let x2 = (x1 - lower_bounds[0])*mults[1];
                    let y2 = (y1 - lower_bounds[1])*mults[3];
                    let z2 = (z1 - lower_bounds[2])*mults[5];

                    x1 *= mults[1];
                    y1 *= mults[3];
                    z1 *= mults[5];
                    
                    output_blocks[(x2 + y2 + z2) as usize] = world.blocks[(x1 + y1 + z1) as usize].clone();
                }

            }
        }
    }

    BlockArray {
        format: world.format,
        dims: output_dims,
        blocks: output_blocks
    }

}

pub fn grow_world (world: BlockArray, new_dims: [i32; 3], merged_world: Vec<Block>) -> BlockArray {
    //Modify this to specify how the fill works (for example adding air to the bottom of a world as opposed to the top)
    
    let mults: Vec<i32> = format_to_mults(world.format.clone(), new_dims);
    let mut output_blocks: Vec<Block> = merged_world.clone();
    
    for x in 0..new_dims[0] {
        for y in 0..new_dims[1] {
            for z in 0..new_dims[2] {

                //Only pushing blocks if they are within the old world size
                if x < world.dims[0] && y < world.dims[1] && z < world.dims[2] { 
                    let x1 = (mults[0]-x).abs()*mults[1];
                    let y1 = (mults[2]-y).abs()*mults[3];
                    let z1 = (mults[4]-z).abs()*mults[5];

                    output_blocks[(x1+y1+z1) as usize] = world.blocks[(x1+y1+z1) as usize].clone()
                }
            }
        }
    }
    
    BlockArray {
        format: world.format,
        dims: new_dims,
        blocks: output_blocks
    }
}

fn format_to_mults (input: [String; 3], dims: [i32; 3]) -> Vec<i32> {

    let mut x_mult: i32 = 1;
    let mut y_mult: i32 = 1;
    let mut z_mult: i32 = 1;
    let mut multiplier = 1;

    let mut x_base: i32 = 0;
    let mut y_base: i32 = 0;
    let mut z_base: i32 = 0;

    for i in 0..3 {
        let chars: Vec<char> = input[i].chars().collect();
        match chars[1] {
            'x' => {
                if chars[0] == '-' { x_base = dims[0] - 1 }
                x_mult = multiplier;
                multiplier *= dims[0];
            },
            'y' => {
                if chars[0] == '-' { y_base = dims[1] - 1 }
                y_mult = multiplier;
                multiplier *= dims[1];
            },
            'z' => {
                if chars[0] == '-' { z_base = dims[2] - 1 }
                z_mult = multiplier;
                multiplier *= dims[2];
            },
            _ => log(1, format!("This should never be seen?? Instead of xyz found {}", chars[1]))
        }
    }

    vec![x_base,x_mult,y_base,y_mult,z_base,z_mult]
}

fn get_block_type (edition: String, version: i32) -> Value {
    match edition.as_str() {
        JAVA_EDITION => Value::UByte(0),
        JAVASCRIPT_EDITION => Value::UByte(0),
        FOURK_EDITION => {
            if version < FOURK_JS {
                Value::UInt(0)
            } else {
                Value::UByte(0)
            }
        },
        _ => {
            log(2, "Unsupported edition detected!");
            log(2, "Assuming default block value??");
            Value::UByte(0)
        }
    }
}

fn record_to_block (indices: Vec<usize>, record: StringRecord, btype: Value, version: i32) -> Option<Block> {
    for i in indices.clone() {
        let lower: i32 = record.get(i).unwrap().parse().unwrap_or(version + 1);
        let upper: i32 = record.get(i+1).unwrap().parse().unwrap_or(0);

        if version < lower || version > upper { continue }

        let raw = [record.get(i+2).unwrap_or_default(), record.get(i+3).unwrap_or_default()];
        match raw_to_block(raw, btype.clone()) {
            Some(b) => return Some(b),
            None => continue
        }

    }
    return None
}

fn raw_to_block (raw: [&str; 2], btype: Value) -> Option<Block> {
    let id = match btype {
        Value::UByte(_) => {
            let val: u8 = match raw[0].parse() {
                Ok(v) => v,
                Err(_) => return None
            };
            Value::UByte(val)
        },
        Value::UInt(_) => {
            let val: u32 = match raw[0].parse() {
                Ok(v) => v,
                Err(_) => return None
            };
            Value::UInt(val)
        },
        _ => return None
    };

    let data: Option<_> = match raw[1] {
        "" => None,
        _ => None //Handle parsing blockdata in the future
    };

    return Some(Block { id: id, block_data: data });
}
