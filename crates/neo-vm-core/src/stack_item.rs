//! Neo VM Stack Item types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stack item types in Neo VM (simplified for zkVM)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[inline]
    pub fn to_bool(&self) -> bool {
        match self {
            StackItem::Null => false,
            StackItem::Boolean(b) => *b,
            StackItem::Integer(i) => *i != 0,
            StackItem::ByteString(b) | StackItem::Buffer(b) => b.iter().any(|&x| x != 0),
            StackItem::Array(a) | StackItem::Struct(a) => !a.is_empty(),
            StackItem::Map(m) => !m.is_empty(),
            StackItem::Pointer(_) => true,
        }
    }

    #[inline]
    pub fn to_integer(&self) -> Option<i128> {
        match self {
            StackItem::Integer(i) => Some(*i),
            StackItem::Boolean(b) => Some(*b as i128),
            _ => None,
        }
    }
}

impl fmt::Display for StackItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackItem::Null => write!(f, "Null"),
            StackItem::Boolean(b) => write!(f, "{}", b),
            StackItem::Integer(i) => write!(f, "{}", i),
            StackItem::ByteString(b) => {
                write!(f, "0x")?;
                for byte in b {
                    write!(f, "{:02x}", byte)?;
                }
                Ok(())
            }
            StackItem::Buffer(b) => write!(f, "Buffer({} bytes)", b.len()),
            StackItem::Array(a) => write!(f, "Array({} items)", a.len()),
            StackItem::Struct(s) => write!(f, "Struct({} fields)", s.len()),
            StackItem::Map(m) => write!(f, "Map({} entries)", m.len()),
            StackItem::Pointer(p) => write!(f, "Pointer({})", p),
        }
    }
}
