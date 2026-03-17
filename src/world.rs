#[derive(Default)]
pub struct World {
    pub edition: String,
    pub version: i32,

    pub world_data: Option<Vec<Tag>>,
    pub player_data: Option<Vec<Tag>>,

    pub is_chunked: bool,
    pub blocks: Option<BlockArray>,
    pub entities: Option<Vec<Entity>>
}

#[derive(Default)]
pub struct Tag {
    pub key: String,
    pub id: String,
    pub value: Value
}

pub enum Value {
    None,
    Byte(i8),
    UnByte(u8),
    Boolean(bool),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<Value>),
    Compound(Box<Tag>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>)
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

#[derive(Default)]
pub struct BlockArray {
    pub format: [String; 3], //The order of iterating through the dimensions, should only contain "+/-" xyz
    pub dims: [i32; 3], //World dimensions in xyz format
    pub blocks: Vec<Block>
}

#[derive(Default)]
pub struct Block {
    pub id: Value,
    pub block_data: Option<Vec<Tag>>
}

#[derive(Default)]
pub struct Entity {
    pub id: Value,
    pub entity_data: Option<Vec<Tag>>
}