use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, num::ParseIntError};

use thiserror::Error;
use regex::Regex;

use crate::functions::{hex2dec,path2reader};

const BLOCK_ID_PATH: &str = "ids/block_ids/";
const BLOCK_ID_FILE: &str = "block_id_file.csv";
const SAMVID_FILE: &str = "ids/samvids.csv";

#[derive(Default)]
pub struct BlockArray {
    x: i32,
    y: i32,
    z: i32,
    blocks: Vec<Block>
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Block {
    byte(u8),
    int(i32),
    string(String)
}

impl Default for Block {
    fn default() -> Self {
        Block::byte(0)
    }
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("File Error: {0}")]
    FileError(#[from] std::io::Error),


    #[error("Parsing Int Error: {0}")]
    ParseIntError(#[from] ParseIntError),

}

pub fn ids_to_blocks (array: Vec<i32>, major_version: &str, minor_version: i32) -> Result<Vec<Block>,ConversionError> {

    let suffix: i32 = samvid_to_suffix(major_version, minor_version)?;

    let path: String = BLOCK_ID_PATH.to_string() + major_version + "_" + &suffix.to_string() + ".csv";
    let reader = path2reader(&path)?;

    let mut id_map: HashMap<i32, Block> = HashMap::new();
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

        if !does_it_exist(pairs[2], minor_version) {
            continue;
        }

        let value: Block = match format.as_str() {
            "byte" => Block::byte(pairs[0].parse()?),
            "int" => Block::int(pairs[0].parse()?),
            "string" => Block::string(pairs[0].to_string()),
            _ => Block::byte(pairs[0].parse()?)
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
            "byte" => Block::byte(pairs[0].parse()?),
            "int" => Block::int(pairs[0].parse()?),
            "string" => Block::string(pairs[0].to_string()),
            _ => Block::byte(pairs[0].parse()?)
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

pub fn rotate_array (input_array: BlockArray, input_format: &[&str; 3], output_format: &[&str; 3]) -> BlockArray {
    let mut output_blocks: Vec<Block> = vec![Block::byte(0); input_array.blocks.len()];
    let dims: &[i32; 3] = &[input_array.x, input_array.y, input_array.z];

    /******************************************
    Format of arrays used in this conversion are as follows:
    [0] = x_mult
    [1] = flipped_x
    [2] = y_mult
    [3] = flipped_y
    [4] = z_mult
    [5] = flipped_z
    ******************************************/

    let in_mults: Vec<i32> = format_to_mults(input_format, dims);
    let out_mults: Vec<i32> = format_to_mults(output_format, dims);

    for x in 0..dims[0] {
        for y in 0..dims[1] {
            for z in 0..dims[2] {
                output_blocks[(((out_mults[1]-1-x).abs()*out_mults[0])+((out_mults[3]-1-y).abs()*out_mults[2])+((out_mults[5]-1-z).abs()*out_mults[4])) as usize] =
                input_array.blocks[(((in_mults[1]-1-x).abs()*in_mults[0])+((in_mults[3]-1-y).abs()*in_mults[2])+((in_mults[5]-1-z).abs()*in_mults[4])) as usize].clone()
            }
        }
    }

    BlockArray {
        x: input_array.x,
        y: input_array.y,
        z: input_array.z,
        blocks: output_blocks
    }
}

fn format_to_mults (input: &[&str; 3], dims: &[i32; 3]) -> Vec<i32> {

    let mut mults: Vec<i32> = vec![1; 6];
    let mut second_level: i32 = 1;

    match input[0].replace("-","").as_str() {
        "x" => second_level = dims[0],
        "y" => second_level = dims[1],
        "z" => second_level = dims[2],
        _ => ()
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

    if input.contains(&"-x") { mults[1] = dims[0]}
    if input.contains(&"-y") { mults[3] = dims[1]}
    if input.contains(&"-z") { mults[5] = dims[2]}

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

fn does_it_exist (valid: &str, version: i32) -> bool {
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

