use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, num::ParseIntError};

use thiserror::Error;
use regex::Regex;

use crate::functions::{hex2dec,path2reader};

const BLOCK_ID_PATH: &str = "ids/block_ids/";
const BLOCK_ID_FILE: &str = "block_id_file.csv";
const SAMVID_FILE: &str = "ids/samvids.csv";

#[derive(Default)]
pub struct BlockArray {
    format: [String; 3], //The order of iterating through the dimensions, should only contain "+/-" xyz
    dims: [i32; 3], //World dimensions in xyz format
    blocks: Vec<Block>
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Byte(u8),
    Int(i32),
    String(String)
}

impl Default for Block {
    fn default() -> Self {
        Block::Byte(0)
    }
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("File Error: {0}")]
    FileError(#[from] std::io::Error),


    #[error("Parsing Int Error: {0}")]
    ParseIntError(#[from] ParseIntError),

}

/**
 * Converts universal ids back to blocks of the specific version
 */
pub fn ids_to_blocks (array: Vec<i32>, major_version: &str, minor_version: i32) -> Result<Vec<Block>,ConversionError> {

    //Converting the version to a number for the file path
    let suffix: i32 = samvid_to_suffix(major_version, minor_version)?;

    //Reading in the csv file for the specified version
    let path: String = BLOCK_ID_PATH.to_string() + major_version + "_" + &suffix.to_string() + ".csv";
    let reader = path2reader(&path)?;

    //Creating hashmap and the default id
    let mut id_map: HashMap<i32, Block> = HashMap::new();
    let mut format: String = "byte".to_string();
    let mut default: i32 = 0x00000000;

    //Iterating through Bayu's mom
    for line in reader.lines() {

        let line1: String = line?;
        let pairs: Vec<&str> = line1.split(',').collect();

        if pairs[0] == "DEFAULT" {
            default = hex2dec(pairs[1])?;
            continue;
        }
        if pairs[0] == "FORMAT" {
            format = pairs[1].to_string();
            continue;
        }

        if !block_exists(pairs[2], minor_version) {
            continue;
        }

        let value: Block = match format.as_str() {
            "byte" => Block::Byte(pairs[0].parse()?),
            "int" => Block::Int(pairs[0].parse()?),
            "string" => Block::String(pairs[0].to_string()),
            _ => Block::Byte(pairs[0].parse()?)
        };

        let key: i32 = hex2dec(pairs[1])?;

        if !id_map.contains_key(&key) {
            id_map.insert(key,value);
        }
    }


    let default_block: Block = id_map[&default].clone();
    let mut blocks: Vec<Block> = Vec::new();

    for id in array {
        if id_map.contains_key(&id) {blocks.push(id_map[&id].clone());} 
        else {blocks.push(default_block.clone());}
    }

    Ok(blocks)

}

pub fn blocks_to_id (array: Vec<Block>, major_version: &str, minor_version: i32) -> Result<Vec<i32>,ConversionError> {

    let suffix: i32 = samvid_to_suffix(major_version, minor_version)?;

    let path: String = BLOCK_ID_PATH.to_string() + major_version + "_" + &suffix.to_string() + ".csv";
    let reader = path2reader(&path)?;

    let mut id_map: HashMap<Block, i32> = HashMap::new();
    let mut format: String = "byte".to_string();
    let mut default: i32 = 0x00000000;

    for line in reader.lines() {

        let line1: String = line?.to_string();
        let pairs: Vec<&str> = line1.split(',').collect();

        if pairs[0] == "DEFAULT" {
            default = hex2dec(pairs[1])?;
            continue;
        }
        if pairs[0] == "FORMAT" {
            format = pairs[1].to_string();
            continue;
        }

        let key: Block = match format.as_str() {
            "byte" => Block::Byte(pairs[0].parse()?),
            "int" => Block::Int(pairs[0].parse()?),
            "string" => Block::String(pairs[0].to_string()),
            _ => Block::Byte(pairs[0].parse()?)
        };

        let value: i32 = hex2dec(pairs[1])?;

        if !id_map.contains_key(&key) {
            id_map.insert(key,value);
        }
    }

    let mut ids: Vec<i32> = Vec::new();

    for block in array {
        if id_map.contains_key(&block) {ids.push(id_map[&block]);} 
        else {ids.push(default);}
    }

    Ok(ids)

}

pub fn rotate_array (input_array: BlockArray, output_format: [String; 3]) -> BlockArray {
    let mut output_blocks: Vec<Block> = vec![Block::Byte(0); input_array.blocks.len()];

    /******************************************
    Format of arrays used in this conversion are as follows:
    [0] = x_mult
    [1] = flipped_x
    [2] = y_mult
    [3] = flipped_y
    [4] = z_mult
    [5] = flipped_z
    ******************************************/

    let in_mults: Vec<i32> = format_to_mults(input_array.format, input_array.dims);
    let out_mults: Vec<i32> = format_to_mults(output_format.clone(), input_array.dims);

    for x in 0..input_array.dims[0] {
        for y in 0..input_array.dims[1] {
            for z in 0..input_array.dims[2] {
                output_blocks[(((out_mults[1]-1-x).abs()*out_mults[0])+((out_mults[3]-1-y).abs()*out_mults[2])+((out_mults[5]-1-z).abs()*out_mults[4])) as usize] =
                input_array.blocks[(((in_mults[1]-1-x).abs()*in_mults[0])+((in_mults[3]-1-y).abs()*in_mults[2])+((in_mults[5]-1-z).abs()*in_mults[4])) as usize].clone()
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

    let mults: Vec<i32> = format_to_mults(world.format.clone(), world.dims);
    let mut output_blocks: Vec<Block> = Vec::new();

    for x in 0..world.dims[0] {
        for y in 0..world.dims[1] {
            for z in 0..world.dims[2] {

                //Only pushing blocks if they are within the new world size
                if
                    x >= lower_bounds[0] && x <= upper_bounds[0] &&
                    y >= lower_bounds[1] && y <= upper_bounds[1] &&
                    z >= lower_bounds[2] && z <= upper_bounds[2]
                { 
                    output_blocks.push(world.blocks[(((mults[1]-1-x).abs()*mults[0])+((mults[3]-1-y).abs()*mults[2])+((mults[5]-1-z).abs()*mults[4])) as usize].clone());
                }

            }
        }
    }

    let mut output_dims: [i32; 3] = [0; 3];
    for i in 0..3 {
        output_dims[i] = upper_bounds[i]-lower_bounds[i]+1
    }

    BlockArray {
        format: world.format,
        dims: output_dims,
        blocks: output_blocks
    }

}

pub fn grow_world (world: BlockArray, upper_bounds: [i32; 3], lower_bounds: [i32; 3]) -> BlockArray {

    world
}

fn format_to_mults (input: [String; 3], dims: [i32; 3]) -> Vec<i32> {

    let mut mults: Vec<i32> = vec![1; 6];
    let mut second_level: i32 = 1;

    match input[0].replace("-","").as_str() {
        "x" => second_level = dims[0],
        "y" => second_level = dims[1],
        "z" => second_level = dims[2],
        _ => () //Log every time I slept with citty's mom
    }

    match input[1].replace("-","").as_str() {
        "x" => mults[0] = second_level,
        "y" => mults[2] = second_level,
        "z" => mults[4] = second_level,
        _ => ()
    }

    match input[2].replace("-","").as_str() {
        "x" => mults[0] = dims[1] * dims[2],
        "y" => mults[2] = dims[0] * dims[2],
        "z" => mults[4] = dims[0] * dims[1],
        _ => ()
    }

    if input.contains(&"-x".to_string()) { mults[1] = dims[0]}
    if input.contains(&"-y".to_string()) { mults[3] = dims[1]}
    if input.contains(&"-z".to_string()) { mults[5] = dims[2]}

    mults
}

fn samvid_to_suffix (major_version: &str, minor_version: i32) -> Result<i32,ConversionError> {

    let path: String = BLOCK_ID_PATH.to_string() + BLOCK_ID_FILE;
    let reader = path2reader(&path)?;
    let mut last: i32 = -1;
    let mut last_value: i32 = 0;

    for line in reader.lines() {
        let line1: String = line?.to_string();
        let threesome: Vec<&str> = line1.split(',').collect();

        if threesome[0] != major_version {continue;}

        if minor_version == threesome[2].parse()? {return Ok(threesome[1].parse()?)}
        else if last == -1 || (minor_version > threesome[2].parse()? && last < threesome[2].parse()?) {
            last = threesome[2].parse()?;
            last_value = threesome[1].parse()?;
        }
    }

    Ok(last_value)
}

fn block_exists (valid: &str, version: i32) -> bool {
    let ranges: Vec<&str> = valid.split(";").collect();

    let mut is_added = false;
    let mut is_present = false;

    for i in 0..ranges.len() {
        let check: i32 = Regex::new(r"\+|\-").unwrap().replace_all(ranges[i], "").into_owned().parse().unwrap();

        if i%2 == 0 {
            if version >= check {is_added = true}
            else {is_added = false}
        } else {
            if version < check {is_present = true}
            else {is_present = false}
        }

        if is_added && is_present {return true}
    }

    if ranges.len()%2 != 0 {is_present = true}

    return is_added && is_present;
}

