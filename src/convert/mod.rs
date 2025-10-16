mod blocks;

use crate::convert::blocks::BlockArray;

pub struct UniversalWorld {
    major_version: String,
    minor_version: i32,
    blocks: BlockArray 
}

