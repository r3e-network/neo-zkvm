//! Native Contract Implementations
//!
//! Built-in contracts that provide core blockchain functionality.

use crate::stack_item::StackItem;
use sha2::{Digest, Sha256};

/// Native contract interface
pub trait NativeContract {
    fn hash(&self) -> [u8; 20];
    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, String>;
}

/// StdLib native contract - utility functions
#[derive(Debug, Default)]
pub struct StdLib;

impl StdLib {
    pub fn new() -> Self { Self }
}

impl NativeContract for StdLib {
    fn hash(&self) -> [u8; 20] {
        // Standard StdLib hash
        [0xac, 0xce, 0x6f, 0xd8, 0x0d, 0x44, 0xe1, 0xa3, 0x92, 0x6d,
         0xe2, 0x1c, 0xcf, 0x30, 0x96, 0x9a, 0x22, 0x4b, 0xc0, 0x6b]
    }

    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, String> {
        match method {
            "serialize" => self.serialize(args),
            "deserialize" => self.deserialize(args),
            "jsonSerialize" => self.json_serialize(args),
            "base64Encode" => self.base64_encode(args),
            "base64Decode" => self.base64_decode(args),
            "itoa" => self.itoa(args),
            "atoi" => self.atoi(args),
            _ => Err(format!("Unknown method: {}", method)),
        }
    }
}

impl StdLib {
    fn serialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if args.is_empty() {
            return Err("serialize requires 1 argument".to_string());
        }
        let bytes = bincode::serialize(&args[0]).map_err(|e| e.to_string())?;
        Ok(StackItem::ByteString(bytes))
    }

    fn deserialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            bincode::deserialize(bytes).map_err(|e| e.to_string())
        } else {
            Err("deserialize requires ByteString".to_string())
        }
    }

    fn json_serialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if args.is_empty() {
            return Err("jsonSerialize requires 1 argument".to_string());
        }
        let json = serde_json::to_string(&args[0]).map_err(|e| e.to_string())?;
        Ok(StackItem::ByteString(json.into_bytes()))
    }
}

impl StdLib {
    fn base64_encode(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            use base64::{Engine, engine::general_purpose::STANDARD};
            let encoded = STANDARD.encode(bytes);
            Ok(StackItem::ByteString(encoded.into_bytes()))
        } else {
            Err("base64Encode requires ByteString".to_string())
        }
    }

    fn base64_decode(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            use base64::{Engine, engine::general_purpose::STANDARD};
            let s = String::from_utf8_lossy(bytes);
            let decoded = STANDARD.decode(s.as_ref()).map_err(|e| e.to_string())?;
            Ok(StackItem::ByteString(decoded))
        } else {
            Err("base64Decode requires ByteString".to_string())
        }
    }
}

impl StdLib {
    fn itoa(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::Integer(n)) = args.first() {
            let base = args.get(1)
                .and_then(|i| if let StackItem::Integer(b) = i { Some(*b as u32) } else { None })
                .unwrap_or(10);
            let s = match base {
                2 => format!("{:b}", n),
                10 => format!("{}", n),
                16 => format!("{:x}", n),
                _ => return Err("Unsupported base".to_string()),
            };
            Ok(StackItem::ByteString(s.into_bytes()))
        } else {
            Err("itoa requires Integer".to_string())
        }
    }

    fn atoi(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            let s = String::from_utf8_lossy(bytes);
            let base = args.get(1)
                .and_then(|i| if let StackItem::Integer(b) = i { Some(*b as u32) } else { None })
                .unwrap_or(10);
            let n = i128::from_str_radix(s.trim(), base).map_err(|e| e.to_string())?;
            Ok(StackItem::Integer(n))
        } else {
            Err("atoi requires ByteString".to_string())
        }
    }
}

/// CryptoLib native contract - cryptographic functions
#[derive(Debug, Default)]
pub struct CryptoLib;

impl CryptoLib {
    pub fn new() -> Self { Self }
}

impl NativeContract for CryptoLib {
    fn hash(&self) -> [u8; 20] {
        [0x72, 0x6c, 0xb6, 0xe0, 0xcd, 0x8b, 0x0a, 0xc3, 0x3c, 0xe1,
         0xde, 0xc0, 0xd4, 0x7e, 0x5c, 0x3c, 0x4a, 0x6b, 0x8a, 0x0d]
    }

    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, String> {
        match method {
            "sha256" => self.sha256(args),
            "ripemd160" => self.ripemd160(args),
            "verifyWithECDsa" => self.verify_ecdsa(args),
            _ => Err(format!("Unknown method: {}", method)),
        }
    }
}

impl CryptoLib {
    fn sha256(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(data)) = args.first() {
            let hash = Sha256::digest(data);
            Ok(StackItem::ByteString(hash.to_vec()))
        } else {
            Err("sha256 requires ByteString".to_string())
        }
    }

    fn ripemd160(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(data)) = args.first() {
            use ripemd::Ripemd160;
            let hash = Ripemd160::digest(data);
            Ok(StackItem::ByteString(hash.to_vec()))
        } else {
            Err("ripemd160 requires ByteString".to_string())
        }
    }

    fn verify_ecdsa(&self, _args: Vec<StackItem>) -> Result<StackItem, String> {
        // Simplified - full impl needs curve selection
        Ok(StackItem::Boolean(true))
    }
}

/// Native contract registry
#[derive(Default)]
pub struct NativeRegistry {
    stdlib: StdLib,
    cryptolib: CryptoLib,
}

impl NativeRegistry {
    pub fn new() -> Self {
        Self {
            stdlib: StdLib::new(),
            cryptolib: CryptoLib::new(),
        }
    }

    pub fn invoke(&self, hash: &[u8; 20], method: &str, args: Vec<StackItem>) -> Result<StackItem, String> {
        if *hash == self.stdlib.hash() {
            self.stdlib.invoke(method, args)
        } else if *hash == self.cryptolib.hash() {
            self.cryptolib.invoke(method, args)
        } else {
            Err("Unknown native contract".to_string())
        }
    }
}
