//! Native Contract Implementations
//!
//! Built-in contracts that provide core blockchain functionality.

use crate::stack_item::StackItem;
use sha2::{Digest, Sha256};

/// Maximum input size for native contract functions (1MB)
const MAX_INPUT_SIZE: usize = 1024 * 1024;

/// Native contract interface
pub trait NativeContract {
    fn hash(&self) -> [u8; 20];
    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, String>;
}

/// StdLib native contract - utility functions
#[derive(Debug, Default)]
pub struct StdLib;

impl StdLib {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    #[inline]
    fn serialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if args.is_empty() {
            return Err("serialize requires 1 argument".to_string());
        }
        let bytes = bincode::serialize(&args[0]).map_err(|e| e.to_string())?;
        Ok(StackItem::ByteString(bytes))
    }

    fn deserialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            if bytes.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "deserialize input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            bincode::deserialize(bytes).map_err(|e| format!("deserialize failed: {}", e))
        } else {
            Err("deserialize requires ByteString argument".to_string())
        }
    }

    #[inline]
    fn json_serialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if args.is_empty() {
            return Err("jsonSerialize requires 1 argument".to_string());
        }
        let json = serde_json::to_string(&args[0]).map_err(|e| e.to_string())?;
        if json.len() > MAX_INPUT_SIZE {
            return Err(format!(
                "jsonSerialize output exceeds maximum size of {} bytes",
                MAX_INPUT_SIZE
            ));
        }
        Ok(StackItem::ByteString(json.into_bytes()))
    }
}

impl StdLib {
    #[inline]
    fn base64_encode(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            if bytes.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "base64Encode input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            use base64::{engine::general_purpose::STANDARD, Engine};
            let encoded = STANDARD.encode(bytes);
            Ok(StackItem::ByteString(encoded.into_bytes()))
        } else {
            Err("base64Encode requires ByteString".to_string())
        }
    }

    #[inline]
    fn base64_decode(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            if bytes.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "base64Decode input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            use base64::{engine::general_purpose::STANDARD, Engine};
            let s = String::from_utf8_lossy(bytes);
            let decoded = STANDARD.decode(s.as_ref()).map_err(|e| e.to_string())?;
            Ok(StackItem::ByteString(decoded))
        } else {
            Err("base64Decode requires ByteString".to_string())
        }
    }
}

impl StdLib {
    #[inline]
    fn itoa(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::Integer(n)) = args.first() {
            let base = args
                .get(1)
                .and_then(|i| {
                    if let StackItem::Integer(b) = i {
                        Some(*b as u32)
                    } else {
                        None
                    }
                })
                .unwrap_or(10);
            if base != 2 && base != 10 && base != 16 {
                return Err(format!(
                    "Unsupported base {}. Supported bases: 2 (binary), 10 (decimal), 16 (hexadecimal)",
                    base
                ));
            }
            let s = match base {
                2 => format!("{:b}", n),
                10 => format!("{}", n),
                16 => format!("{:x}", n),
                _ => unreachable!(),
            };
            Ok(StackItem::ByteString(s.into_bytes()))
        } else {
            Err("itoa requires Integer".to_string())
        }
    }

    #[inline]
    fn atoi(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            if bytes.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "atoi input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            let s = String::from_utf8_lossy(bytes);
            let base = args
                .get(1)
                .and_then(|i| {
                    if let StackItem::Integer(b) = i {
                        Some(*b as u32)
                    } else {
                        None
                    }
                })
                .unwrap_or(10);
            if base != 2 && base != 10 && base != 16 {
                return Err(format!(
                    "Unsupported base {}. Supported bases: 2 (binary), 10 (decimal), 16 (hexadecimal)",
                    base
                ));
            }
            let n = i128::from_str_radix(s.trim(), base).map_err(|e| e.to_string())?;
            Ok(StackItem::Integer(n))
        } else {
            Err("atoi requires ByteString".to_string())
        }
    }
}

impl NativeContract for StdLib {
    #[inline]
    fn hash(&self) -> [u8; 20] {
        [
            0xac, 0xce, 0x6f, 0xd8, 0x0d, 0x44, 0xe1, 0xa3, 0x92, 0x6d, 0xe2, 0x1c, 0xcf, 0x30,
            0x96, 0x9a, 0x22, 0x4b, 0xc0, 0x6b,
        ]
    }

    #[inline]
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

/// CryptoLib native contract - cryptographic functions
#[derive(Debug, Default)]
pub struct CryptoLib;

impl CryptoLib {
    pub fn new() -> Self {
        Self
    }
}

impl NativeContract for CryptoLib {
    #[inline]
    fn hash(&self) -> [u8; 20] {
        [
            0x72, 0x6c, 0xb6, 0xe0, 0xcd, 0x8b, 0x0a, 0xc3, 0x3c, 0xe1, 0xde, 0xc0, 0xd4, 0x7e,
            0x5c, 0x3c, 0x4a, 0x6b, 0x8a, 0x0d,
        ]
    }

    #[inline]
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
    #[inline]
    fn sha256(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(data)) = args.first() {
            if data.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "sha256 input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            let hash = Sha256::digest(data);
            Ok(StackItem::ByteString(hash.to_vec()))
        } else {
            Err("sha256 requires ByteString".to_string())
        }
    }

    #[inline]
    fn ripemd160(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(data)) = args.first() {
            if data.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "ripemd160 input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            use ripemd::Ripemd160;
            let hash = Ripemd160::digest(data);
            Ok(StackItem::ByteString(hash.to_vec()))
        } else {
            Err("ripemd160 requires ByteString".to_string())
        }
    }

    #[inline]
    fn verify_ecdsa(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

        if args.len() < 2 {
            return Err("verify_ecdsa requires at least 2 arguments".to_string());
        }

        let message = match &args[0] {
            StackItem::ByteString(msg) => msg.as_slice(),
            _ => return Err("verify_ecdsa: first argument must be ByteString".to_string()),
        };

        let signature = match &args[1] {
            StackItem::ByteString(sig) => sig.as_slice(),
            _ => return Err("verify_ecdsa: second argument must be ByteString".to_string()),
        };

        let pubkey = if args.len() >= 3 {
            match &args[2] {
                StackItem::ByteString(pk) => pk.as_slice(),
                _ => return Err("verify_ecdsa: third argument must be ByteString".to_string()),
            }
        } else {
            return Err("verify_ecdsa: public key required".to_string());
        };

        if message.len() > MAX_INPUT_SIZE {
            return Err(format!(
                "verify_ecdsa message exceeds maximum size of {} bytes",
                MAX_INPUT_SIZE
            ));
        }

        let signature = Signature::from_slice(signature)
            .map_err(|_| "Invalid ECDSA signature format".to_string())?;
        let verifying_key = VerifyingKey::from_sec1_bytes(pubkey)
            .map_err(|_| "Invalid public key format".to_string())?;

        Ok(StackItem::Boolean(
            verifying_key.verify(message, &signature).is_ok(),
        ))
    }
}

/// Native contract registry
#[derive(Default)]
pub struct NativeRegistry {
    stdlib: StdLib,
    cryptolib: CryptoLib,
}

impl NativeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            stdlib: StdLib::new(),
            cryptolib: CryptoLib::new(),
        }
    }

    #[inline]
    pub fn invoke(
        &self,
        hash: &[u8; 20],
        method: &str,
        args: Vec<StackItem>,
    ) -> Result<StackItem, String> {
        if *hash == self.stdlib.hash() {
            self.stdlib.invoke(method, args)
        } else if *hash == self.cryptolib.hash() {
            self.cryptolib.invoke(method, args)
        } else {
            Err("Unknown native contract".to_string())
        }
    }
}
