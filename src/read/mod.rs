use std::path::PathBuf;
use thiserror::Error;

use mc_classic::{self, Level, read_level};
use mc_classic_js;

use crate::world::{self, Block, BlockArray, Tag, Value, World};
use crate::log::log;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error(transparent)]
    ClassicReadError(#[from] mc_classic::ClassicError)
}

pub fn read_classic_file (path: PathBuf) -> Result<World,ReadError> {

    log(String::from("Reading Classic World..."));

    let level: Level = match read_level(path) {
        Ok(value) => value,
        Err(error) => {
            log(String::from("File Not Opened"));
            return Err(ReadError::ClassicReadError(error))
        }
    };

    let mut world_data: Vec<Tag> = Vec::new();

    //World data should be handled more dynamically in the library
    //Currently the static implementation means that world data must be handled as such

    if level.cloudColor.is_some() {world_data.push(Tag{key: "cloudColor".to_string(), id: String::from("Int"), value: Value::Int(level.cloudColor.unwrap())})}
    if level.rotSpawn.is_some() {world_data.push(Tag{key: "rotSpawn".to_string(), id: String::from("Float"), value: Value::Float(level.rotSpawn.unwrap())})}
    if level.tickCount.is_some() {world_data.push(Tag{key: "tickCount".to_string(), id: String::from("Int"), value: Value::Int(level.tickCount.unwrap())})}
    if level.unprocessed.is_some() {world_data.push(Tag{key: "unprocessed".to_string(), id: String::from("Int"), value: Value::Int(level.unprocessed.unwrap())})}
    if level.xSpawn.is_some() {world_data.push(Tag{key: "xSpawn".to_string(), id: String::from("Int"), value: Value::Int(level.xSpawn.unwrap())})}
    if level.ySpawn.is_some() {world_data.push(Tag{key: "ySpawn".to_string(), id: String::from("Int"), value: Value::Int(level.ySpawn.unwrap())})}
    if level.zSpawn.is_some() {world_data.push(Tag{key: "zSpawn".to_string(), id: String::from("Int"), value: Value::Int(level.zSpawn.unwrap())})}
    if level.networkMode.is_some() {world_data.push(Tag{key: "networkMode".to_string(), id: String::from("Boolean"), value: Value::Boolean(level.networkMode.unwrap())})}
    if level.fogColor.is_some() {world_data.push(Tag{key: "fogColor".to_string(), id: String::from("Int"), value: Value::Int(level.fogColor.unwrap())})}
    if level.skyColor.is_some() {world_data.push(Tag{key: "skyColor".to_string(), id: String::from("Int"), value: Value::Int(level.skyColor.unwrap())})}
    if level.waterLevel.is_some() {world_data.push(Tag{key: "waterLevel".to_string(), id: String::from("Int"), value: Value::Int(level.waterLevel.unwrap())})}
    if level.creativeMode.is_some() {world_data.push(Tag{key: "creativeMode".to_string(), id: String::from("Boolean"), value: Value::Boolean(level.creativeMode.unwrap())})}
    if level.growTrees.is_some() {world_data.push(Tag{key: "growTrees".to_string(), id: String::from("Boolean"), value: Value::Boolean(level.growTrees.unwrap())})}
    if level.createTime.is_some() {world_data.push(Tag{key: "createTime".to_string(), id: String::from("Long"), value: Value::Long(level.createTime.unwrap())})}
    if level.name.is_some() {world_data.push(Tag{key: "name".to_string(), id: String::from("String"), value: Value::String(level.name.unwrap())})}
    if level.creator.is_some() {world_data.push(Tag{key: "creator".to_string(), id: String::from("String"), value: Value::String(level.creator.unwrap())})}

    //Handle Playerdata
    //Handle Entities

    let mut blocks = BlockArray::default();

    blocks.format = [
        String::from("+x"),
        String::from("+z"),
        String::from("+y"),
        ];

    //if samvid is in pre-classic
    //block.dims = [256,256,64]
    blocks.dims = [
        level.width.unwrap_or_default(),
        level.depth.unwrap_or_default(),
        level.height.unwrap_or_default(),
        ];

    if level.blocks.is_some() {
        let mut block_arr: Vec<Block> = Vec::new();
        for block in level.blocks.unwrap() {
            block_arr.push(
                Block { 
                    id: Value::UnByte(block),
                    block_data: None
                });
        }
    }

    let mut world = World::default();

    //Set Edition
    //Set version

    world.world_data = Some(world_data);
    //world.player_data = Some(player_data);

    world.blocks = Some(blocks);

    return Ok(world);

}