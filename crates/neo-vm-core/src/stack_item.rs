//! Neo VM Stack Item types

use serde::{Deserialize, Serialize};

/// Stack item types in Neo VM (simplified for zkVM)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackItem {
    Null,
    Boolean(bool),
    Integer(i128),
    ByteString(Vec<u8>),
    Buffer(Vec<u8>),
    Array(Vec<StackItem>),
    Struct(Vec<StackItem>),
    Map(Vec<(StackItem, StackItem)>),
    Pointer(u32),
}

impl StackItem {
    pub fn to_bool(&self) -> bool {
        match self {
            StackItem::Null => false,
            StackItem::Boolean(b) => *b,
            StackItem::Integer(i) => *i != 0,
            StackItem::ByteString(b) | StackItem::Buffer(b) => b.iter().any(|&x| x != 0),
            StackItem::Array(a) | StackItem::Struct(a) => !a.is_empty(),
            StackItem::Map(m) => !m.is_empty(),
            _ => true,
        }
    }

    pub fn to_integer(&self) -> Option<i128> {
        match self {
            StackItem::Integer(i) => Some(*i),
            StackItem::Boolean(b) => Some(*b as i128),
            _ => None,
        }
    }
}
