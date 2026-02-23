//! Neo VM Stack Item types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stack item types in Neo VM (simplified for zkVM).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
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
    /// Convert this item to a boolean following Neo N3 truthiness rules.
    #[inline]
    #[must_use]
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

    /// Convert to integer following Neo N3 semantics.
    ///
    /// - `Integer` → value directly
    /// - `Boolean` → 0 or 1
    /// - `ByteString`/`Buffer` → little-endian two's complement (up to 16 bytes)
    #[inline]
    #[must_use]
    pub fn to_integer(&self) -> Option<i128> {
        match self {
            StackItem::Integer(i) => Some(*i),
            StackItem::Boolean(b) => Some(*b as i128),
            StackItem::ByteString(b) | StackItem::Buffer(b) => {
                if b.is_empty() {
                    return Some(0);
                }
                if b.len() > 16 {
                    return None;
                }
                // Neo N3: little-endian two's complement
                let mut padded = [0u8; 16];
                let sign_extend = if b[b.len() - 1] & 0x80 != 0 {
                    0xFF
                } else {
                    0x00
                };
                padded.iter_mut().for_each(|p| *p = sign_extend);
                padded[..b.len()].copy_from_slice(b);
                Some(i128::from_le_bytes(padded))
            }
            _ => None,
        }
    }

    /// Extract byte content from ByteString or Buffer variants.
    #[inline]
    #[must_use]
    pub fn to_bytes(&self) -> Option<&[u8]> {
        match self {
            StackItem::ByteString(b) | StackItem::Buffer(b) => Some(b),
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
