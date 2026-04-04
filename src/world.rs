use std::collections::HashMap;

use serde_json::Value as JValue;

use crate::log::log;

#[derive(Default,Clone)]
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

impl Generic for u64 {
    fn set (&self) -> Value {
        Value::ULong(*self)
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
    ULong(u64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<Value>),
    Compound(Box<HashMap<String,Value>>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    Null
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl Value {
    pub fn new(generic: impl Generic) -> Value {
        generic.set()
    }

    pub fn json_to_value(jvalue: JValue) -> Value {
        match jvalue {
            JValue::Bool(b) => Value::Boolean(b),
            JValue::Number(n) => {
                if n.is_i64() { return Value::Long(n.as_i64().unwrap()) }
                else if n.is_u64() { return Value::ULong(n.as_u64().unwrap()) }
                else if n.is_f64() { return Value::Double(n.as_f64().unwrap()) }
                else { return Value::Null }
            },
            JValue::String(s) => Value::String(s),
            JValue::Array(a) => {
                let mut new: Vec<Value> = Vec::new();
                for v in a {new.push(Value::json_to_value(v))}
                return Value::List(new)
            },
            JValue::Object(o) => {
                let mut new: HashMap<String, Value> = HashMap::new();
                for (key, value) in o {
                    new.insert(key, Value::json_to_value(value));
                }
                return Value::Compound(Box::new(new))
            },
            JValue::Null => Value::Null,
            _ => {
                log(-1, "Unknown json type - library update might have broken something");
                Value::Null
            }
        }
    }

    pub fn as_u8 (&self) -> Option<u8> {
        match self {
            Value::UByte(u) => Some(*u),
            _ => None
        }
    }

    pub fn as_i8 (&self) -> Option<i8> {
        match self {
            Value::Byte(i) => Some(*i),
            _ => None
        }
    }
    
    pub fn as_bool (&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None
        }
    }

    pub fn as_i16 (&self) -> Option<i16> {
        match self {
            Value::Short(i) => Some(*i),
            _ => None
        }
    }

    pub fn as_i32 (&self) -> Option<i32> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None
        }
    }

    pub fn as_i64 (&self) -> Option<i64> {
        match self {
            Value::Long(i) => Some(*i),
            _ => None
        }
    }

    pub fn as_u64 (&self) -> Option<u64> {
        match self {
            Value::ULong(u) => Some(*u),
            _ => None
        }
    }

    pub fn as_f32 (&self) -> Option<f32> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None
        }
    }

    pub fn as_f64 (&self) -> Option<f64> {
        match self {
            Value::Double(f) => Some(*f),
            _ => None
        }
    }

    pub fn as_string (&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None
        }
    }

    pub fn as_i8_vec (&self) -> Option<Vec<i8>> {
        match self {
            Value::ByteArray(b) => Some(b.clone()),
            _ => None
        }
    }

    pub fn as_i32_vec (&self) -> Option<Vec<i32>> {
        match self {
            Value::IntArray(i) => Some(i.clone()),
            _ => None
        }
    }

    pub fn as_i64_vec (&self) -> Option<Vec<i64>> {
        match self {
            Value::LongArray(l) => Some(l.clone()),
            _ => None
        }
    }
}

#[derive(Clone)]
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

#[derive(Default, Clone)]
pub struct Block {
    pub id: Value,
    pub block_data: Option<HashMap<String,Value>>
}

#[derive(Default, Clone)]
pub struct Entity {
    pub id: Value,
    pub entity_data: Option<HashMap<String,Value>>
}