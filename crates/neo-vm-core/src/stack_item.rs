//! Neo VM Stack Item types

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

/// Stack item types in Neo VM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackItem {
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Integer (arbitrary precision)
    Integer(BigInt),
    /// Byte array
    ByteString(Vec<u8>),
    /// Buffer (mutable byte array)
    Buffer(Vec<u8>),
    /// Array of stack items
    Array(Vec<StackItem>),
    /// Struct (value-type array)
    Struct(Vec<StackItem>),
    /// Map
    Map(Vec<(StackItem, StackItem)>),
    /// Pointer
    Pointer(u32),
    /// Interop interface
    InteropInterface(u64),
}

impl StackItem {
    /// Convert to boolean
    pub fn to_bool(&self) -> bool {
        match self {
            StackItem::Null => false,
            StackItem::Boolean(b) => *b,
            StackItem::Integer(i) => *i != BigInt::from(0),
            StackItem::ByteString(b) | StackItem::Buffer(b) => {
                b.iter().any(|&x| x != 0)
            }
            StackItem::Array(a) | StackItem::Struct(a) => !a.is_empty(),
            StackItem::Map(m) => !m.is_empty(),
            _ => true,
        }
    }

    /// Convert to integer
    pub fn to_integer(&self) -> Option<BigInt> {
        match self {
            StackItem::Integer(i) => Some(i.clone()),
            StackItem::Boolean(b) => Some(BigInt::from(*b as i32)),
            StackItem::ByteString(b) | StackItem::Buffer(b) => {
                Some(BigInt::from_signed_bytes_le(b))
            }
            _ => None,
        }
    }
}
