use std::collections::HashMap;

#[derive(Default)]
pub struct World {
    pub edition: String,
    pub version: i32,

    pub world_data: Option<HashMap<String,Value>>,
    pub player_data: Option<HashMap<String,Value>>,

    pub is_chunked: bool,
    pub blocks: Option<BlockArray>,
    pub entities: Option<Vec<Entity>>
}

pub trait Generic {
    fn set (&self) -> Value;
}

impl Generic for i8 {
    fn set (&self) -> Value {
        Value::Byte(*self)
    }
}

impl Generic for u8 {
    fn set (&self) -> Value {
        Value::UByte(*self)
    }
}

impl Generic for bool {
    fn set (&self) -> Value {
        Value::Boolean(*self)
    }
}

impl Generic for i16 {
    fn set (&self) -> Value {
        Value::Short(*self)
    }
}

impl Generic for i32 {
    fn set (&self) -> Value {
        Value::Int(*self)
    }
}

impl Generic for i64 {
    fn set (&self) -> Value {
        Value::Long(*self)
    }
}

impl Generic for f32 {
    fn set (&self) -> Value {
        Value::Float(*self)
    }
}

impl Generic for f64 {
    fn set (&self) -> Value {
        Value::Double(*self)
    }
}

impl Generic for String {
    fn set (&self) -> Value {
        Value::String(self.to_string())
    }
}

impl Generic for Vec<Value> {
    fn set (&self) -> Value {
        Value::List(self.to_vec())
    }
}

impl Generic for Box<HashMap<String,Value>> {
    fn set (&self) -> Value {
        Value::Compound(self.clone())
    }
}

impl Generic for Vec<i8> {
    fn set (&self) -> Value {
        Value::ByteArray(self.to_vec())
    }
}

impl Generic for Vec<i32> {
    fn set (&self) -> Value {
        Value::IntArray(self.to_vec())
    }
}

impl Generic for Vec<i64> {
    fn set (&self) -> Value {
        Value::LongArray(self.to_vec())
    }
}



#[derive(Clone)]
pub enum Value {
    Byte(i8),
    UByte(u8),
    Boolean(bool),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<Value>),
    Compound(Box<HashMap<String,Value>>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>)
}

impl Default for Value {
    fn default() -> Self {
        Value::Byte(0)
    }
}

impl Value {
    pub fn new(generic: impl Generic) -> Value {
        generic.set()
    }
}

pub struct BlockArray {
    pub format: [String; 3], //The order of iterating through the dimensions, should only contain "+/-" xyz
    pub dims: [i32; 3], //World dimensions in xyz format
    pub blocks: Vec<Block>
}

impl Default for BlockArray {
    fn default() -> Self {
        BlockArray { 
            format: ["+x".to_string(),"+y".to_string(),"+z".to_string()],
            dims: [0; 3],
            blocks: Vec::default()
        }
    }
}

#[derive(Default)]
pub struct Block {
    pub id: Value,
    pub block_data: Option<HashMap<String,Value>>
}

#[derive(Default)]
pub struct Entity {
    pub id: Value,
    pub entity_data: Option<HashMap<String,Value>>
}