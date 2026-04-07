use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;

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

impl Generic for bool {
    fn set (&self) -> Value {
        Value::Boolean(*self)
    }
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

impl Generic for i16 {
    fn set (&self) -> Value {
        Value::Short(*self)
    }
}

impl Generic for u16 {
    fn set (&self) -> Value {
        Value::UShort(*self)
    }
}

impl Generic for i32 {
    fn set (&self) -> Value {
        Value::Int(*self)
    }
}

impl Generic for u32 {
    fn set (&self) -> Value {
        Value::UInt(*self)
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



#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Byte(i8),
    UByte(u8),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Boolean(b1) => {
                match other {
                    Value::Boolean(b2) => b1 == b2,
                    _ => false
                }
            },
            Value::Byte(b1) => {
                match other {
                    Value::Byte(b2) => b1 == b2,
                    _ => false
                }
            },
            Value::UByte(b1) => {
                match other {
                    Value::UByte(b2) => b1 == b2,
                    _ => false
                }
            },
            Value::Short(s1) => {
                match other {
                    Value::Short(s2) => s1 == s2,
                    _ => false
                }
            },
            Value::UShort(s1) => {
                match other {
                    Value::UShort(s2) => s1 == s2,
                    _ => false
                }
            },
            Value::Int(i1) => {
                match other {
                    Value::Int(i2) => i1 == i2,
                    _ => false
                }
            },
            Value::UInt(i1) => {
                match other {
                    Value::UInt(i2) => i1 == i2,
                    _ => false
                }
            },
            Value::Long(l1) => {
                match other {
                    Value::Long(l2) => l1 == l2,
                    _ => false
                }
            },
            Value::ULong(l1) => {
                match other {
                    Value::ULong(l2) => l1 == l2,
                    _ => false
                }
            },
            Value::Float(f1) => {
                match other {
                    Value::Float(f2) => f1 == f2,
                    _ => false
                }
            },
            Value::Double(f1) => {
                match other {
                    Value::Double(f2) => f1 == f2,
                    _ => false
                }
            },
            Value::String(s1) => {
                match other {
                    Value::String(s2) => s1 == s2,
                    _ => false
                }
            },
            Value::List(list1) => {
                match other {
                    Value::List(list2) => list1 == list2,
                    _ => false
                }
            },
            Value::Compound(map1) => { 
                match other {
                    Value::Compound(map2) => {
                        let mut pairs1: Vec<_> = map1.iter().collect();
                        pairs1.sort_by_key(|&(k, _v)| k);
                        let mut pairs2: Vec<_> = map1.iter().collect();
                        pairs2.sort_by_key(|&(k, _v)| k);
                        return pairs1 == pairs2  
                    },
                    _ => false
                }
            },
            Value::ByteArray(list1) => {
                match other {
                    Value::ByteArray(list2) => list1 == list2,
                    _ => false
                }
            },
            Value::IntArray(list1) => {
                match other {
                    Value::IntArray(list2) => list1 == list2,
                    _ => false
                }
            },
            Value::LongArray(list1) => {
                match other {
                    Value::LongArray(list2) => list1 == list2,
                    _ => false
                }
            },
            Value::Null => {
                match other {
                    Value::Null => true,
                    _ => false
                }
            },
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
        
        match self {
            Value::Boolean(b) => b.hash(state),
            Value::Byte(b) => b.hash(state),
            Value::UByte(b) => b.hash(state),
            Value::Short(s) => s.hash(state),
            Value::UShort(s) => s.hash(state),
            Value::Int(i) => i.hash(state),
            Value::UInt(i) => i.hash(state),
            Value::Long(l) => l.hash(state),
            Value::ULong(l) => l.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::Double(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::List(list) => list.hash(state),
            Value::Compound(map) => {
                let mut pairs: Vec<_> = map.iter().collect();
                pairs.sort_by_key(|&(k, _v)| k);
                for (key, value) in pairs {
                    key.hash(state);
                    value.hash(state);
                }
            },
            Value::ByteArray(list) => list.hash(state),
            Value::IntArray(list) => list.hash(state),
            Value::LongArray(list) => list.hash(state),
            Value::Null => (),
        }

        let _ = state.finish();
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

    pub fn type_as_str (&self) -> &str {
        match self {
            Value::Boolean(_) => "bool",
            Value::Byte(_) => "byte",
            Value::UByte(_) => "ubyte",
            Value::Short(_) => "short",
            Value::UShort(_) => "ushort",
            Value::Int(_) => "int",
            Value::UInt(_) => "uint",
            Value::Long(_) => "long",
            Value::ULong(_) => "ulong",
            Value::Float(_) => "float",
            Value::Double(_) => "double",
            Value::String(_) => "string",
            Value::List(values) => "list",
            Value::Compound(hash_map) => "compound",
            Value::ByteArray(items) => "byte_array",
            Value::IntArray(items) => "int_array",
            Value::LongArray(items) => "long_array",
            Value::Null => "null",
        }
    }

    pub fn as_bool (&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None
        }
    }

    pub fn as_i8 (&self) -> Option<i8> {
        match self {
            Value::Byte(b) => Some(*b),
            Value::UByte(b) => Some(*b as i8),
            Value::Short(s) => Some(*s as i8),
            Value::UShort(s) => Some(*s as i8),
            Value::Int(i) => Some(*i as i8),
            Value::UInt(i) => Some(*i as i8),
            Value::Long(l) => Some(*l as i8),
            Value::ULong(l) => Some(*l as i8),
            Value::Float(f) => Some(*f as i8),
            Value::Double(f) => Some(*f as i8),
            _ => None
        }
    }

    pub fn as_u8 (&self) -> Option<u8> {
        match self {
            Value::Byte(b) => Some(*b as u8),
            Value::UByte(b) => Some(*b),
            Value::Short(s) => Some(*s as u8),
            Value::UShort(s) => Some(*s as u8),
            Value::Int(i) => Some(*i as u8),
            Value::UInt(i) => Some(*i as u8),
            Value::Long(l) => Some(*l as u8),
            Value::ULong(l) => Some(*l as u8),
            Value::Float(f) => Some(*f as u8),
            Value::Double(f) => Some(*f as u8),
            _ => None
        }
    }

    pub fn as_i16 (&self) -> Option<i16> {
        match self {
            Value::Byte(b) => Some(*b as i16),
            Value::UByte(b) => Some(*b as i16),
            Value::Short(s) => Some(*s),
            Value::UShort(s) => Some(*s as i16),
            Value::Int(i) => Some(*i as i16),
            Value::UInt(i) => Some(*i as i16),
            Value::Long(l) => Some(*l as i16),
            Value::ULong(l) => Some(*l as i16),
            Value::Float(f) => Some(*f as i16),
            Value::Double(f) => Some(*f as i16),
            _ => None
        }
    }

    pub fn as_u16 (&self) -> Option<u16> {
        match self {
            Value::Byte(b) => Some(*b as u16),
            Value::UByte(b) => Some(*b as u16),
            Value::Short(s) => Some(*s as u16),
            Value::UShort(s) => Some(*s),
            Value::Int(i) => Some(*i as u16),
            Value::UInt(i) => Some(*i as u16),
            Value::Long(l) => Some(*l as u16),
            Value::ULong(l) => Some(*l as u16),
            Value::Float(f) => Some(*f as u16),
            Value::Double(f) => Some(*f as u16),
            _ => None
        }
    }

    pub fn as_i32 (&self) -> Option<i32> {
        match self {
            Value::Byte(b) => Some(*b as i32),
            Value::UByte(b) => Some(*b as i32),
            Value::Short(s) => Some(*s as i32),
            Value::UShort(s) => Some(*s as i32),
            Value::Int(i) => Some(*i),
            Value::UInt(i) => Some(*i as i32),
            Value::Long(l) => Some(*l as i32),
            Value::ULong(l) => Some(*l as i32),
            Value::Float(f) => Some(*f as i32),
            Value::Double(f) => Some(*f as i32),
            _ => None
        }
    }

    pub fn as_u32 (&self) -> Option<u32> {
        match self {
            Value::Byte(b) => Some(*b as u32),
            Value::UByte(b) => Some(*b as u32),
            Value::Short(s) => Some(*s as u32),
            Value::UShort(s) => Some(*s as u32),
            Value::Int(i) => Some(*i as u32),
            Value::UInt(i) => Some(*i),
            Value::Long(l) => Some(*l as u32),
            Value::ULong(l) => Some(*l as u32),
            Value::Float(f) => Some(*f as u32),
            Value::Double(f) => Some(*f as u32),
            _ => None
        }
    }

    pub fn as_i64 (&self) -> Option<i64> {
        match self {
            Value::Byte(b) => Some(*b as i64),
            Value::UByte(b) => Some(*b as i64),
            Value::Short(s) => Some(*s as i64),
            Value::UShort(s) => Some(*s as i64),
            Value::Int(i) => Some(*i as i64),
            Value::UInt(i) => Some(*i as i64),
            Value::Long(l) => Some(*l),
            Value::ULong(l) => Some(*l as i64),
            Value::Float(f) => Some(*f as i64),
            Value::Double(f) => Some(*f as i64),
            _ => None
        }
    }

    pub fn as_u64 (&self) -> Option<u64> {
        match self {
            Value::Byte(b) => Some(*b as u64),
            Value::UByte(b) => Some(*b as u64),
            Value::Short(s) => Some(*s as u64),
            Value::UShort(s) => Some(*s as u64),
            Value::Int(i) => Some(*i as u64),
            Value::UInt(i) => Some(*i as u64),
            Value::Long(l) => Some(*l as u64),
            Value::ULong(l) => Some(*l),
            Value::Float(f) => Some(*f as u64),
            Value::Double(f) => Some(*f as u64),
            _ => None
        }
    }

    pub fn as_f32 (&self) -> Option<f32> {
        match self {
            Value::Byte(b) => Some(*b as f32),
            Value::UByte(b) => Some(*b as f32),
            Value::Short(s) => Some(*s as f32),
            Value::UShort(s) => Some(*s as f32),
            Value::Int(i) => Some(*i as f32),
            Value::UInt(i) => Some(*i as f32),
            Value::Long(l) => Some(*l as f32),
            Value::ULong(l) => Some(*l as f32),
            Value::Float(f) => Some(*f),
            Value::Double(f) => Some(*f as f32),
            _ => None
        }
    }

    pub fn as_f64 (&self) -> Option<f64> {
        match self {
            Value::Byte(b) => Some(*b as f64),
            Value::UByte(b) => Some(*b as f64),
            Value::Short(s) => Some(*s as f64),
            Value::UShort(s) => Some(*s as f64),
            Value::Int(i) => Some(*i as f64),
            Value::UInt(i) => Some(*i as f64),
            Value::Long(l) => Some(*l as f64),
            Value::ULong(l) => Some(*l as f64),
            Value::Float(f) => Some(*f as f64),
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

#[derive(Clone, Debug)]
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

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub id: Value,
    pub block_data: Option<HashMap<String,Value>>
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);

        if self.block_data.clone().is_some() {
            let data = self.block_data.clone().unwrap();
            let mut pairs: Vec<_> = data.iter().collect();
            pairs.sort_by_key(|&(k, _v)| k);
            for (key, value) in pairs {
                key.hash(state);
                value.hash(state);
            }
        }

        let _ = state.finish();
        
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: Value,
    pub entity_data: Option<HashMap<String,Value>>
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);

        if self.entity_data.clone().is_some() {
            let data = self.entity_data.clone().unwrap();
            let mut pairs: Vec<_> = data.iter().collect();
            pairs.sort_by_key(|&(k, _v)| k);
            for (key, value) in pairs {
                key.hash(state);
                value.hash(state);
            }
        }

        let _ = state.finish();
        
    }
}