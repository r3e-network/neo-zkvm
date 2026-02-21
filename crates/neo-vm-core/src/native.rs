//! Native Contract Implementations
//!
//! Built-in contracts that provide core blockchain functionality.

use crate::stack_item::StackItem;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Maximum input size for native contract functions (1MB)
const MAX_INPUT_SIZE: usize = 1024 * 1024;

/// Errors from native contract invocations
#[derive(Error, Debug, PartialEq)]
pub enum NativeContractError {
    #[error("Unknown method '{method}' on contract {contract}")]
    UnknownMethod {
        contract: &'static str,
        method: String,
    },
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Input exceeds maximum size of {max} bytes")]
    InputTooLarge { max: usize },
    #[error("{0}")]
    Other(String),
}

/// Native contract interface
pub trait NativeContract {
    fn hash(&self) -> [u8; 20];
    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, NativeContractError>;
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

    fn json_deserialize(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if let Some(StackItem::ByteString(bytes)) = args.first() {
            if bytes.len() > MAX_INPUT_SIZE {
                return Err(format!(
                    "jsonDeserialize input exceeds maximum size of {} bytes",
                    MAX_INPUT_SIZE
                ));
            }
            let s = String::from_utf8(bytes.to_vec())
                .map_err(|e| format!("jsonDeserialize: invalid UTF-8: {}", e))?;
            let item: StackItem = serde_json::from_str(&s)
                .map_err(|e| format!("jsonDeserialize: invalid JSON: {}", e))?;
            Ok(item)
        } else {
            Err("jsonDeserialize requires ByteString argument".to_string())
        }
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
            let s = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
            let decoded = STANDARD.decode(s.as_str()).map_err(|e| e.to_string())?;
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
            let s = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
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
    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, NativeContractError> {
        let map_err = |r: Result<StackItem, String>| r.map_err(NativeContractError::Other);
        match method {
            "serialize" => map_err(self.serialize(args)),
            "deserialize" => map_err(self.deserialize(args)),
            "jsonSerialize" => map_err(self.json_serialize(args)),
            "jsonDeserialize" => map_err(self.json_deserialize(args)),
            "base64Encode" => map_err(self.base64_encode(args)),
            "base64Decode" => map_err(self.base64_decode(args)),
            "itoa" => map_err(self.itoa(args)),
            "atoi" => map_err(self.atoi(args)),
            _ => Err(NativeContractError::UnknownMethod {
                contract: "StdLib",
                method: method.to_string(),
            }),
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
    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, NativeContractError> {
        let map_err = |r: Result<StackItem, String>| r.map_err(NativeContractError::Other);
        match method {
            "sha256" => map_err(self.sha256(args)),
            "ripemd160" => map_err(self.ripemd160(args)),
            "verifyWithECDsa" => map_err(self.verify_ecdsa(args)),
            "checkSig" => map_err(self.check_sig(args)),
            "murmur32" => map_err(self.murmur32(args)),
            _ => Err(NativeContractError::UnknownMethod {
                contract: "CryptoLib",
                method: method.to_string(),
            }),
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

    /// Verify a single ECDSA signature with a simplified API.
    ///
    /// Arguments: (message: ByteString, signature: ByteString, pubkey: ByteString)
    /// Returns: Boolean indicating whether the signature is valid.
    #[inline]
    fn check_sig(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if args.len() < 3 {
            return Err("checkSig requires 3 arguments: message, signature, pubkey".to_string());
        }

        let message = match &args[0] {
            StackItem::ByteString(msg) => msg.as_slice(),
            _ => return Err("checkSig: first argument (message) must be ByteString".to_string()),
        };

        let signature = match &args[1] {
            StackItem::ByteString(sig) => sig.as_slice(),
            _ => return Err("checkSig: second argument (signature) must be ByteString".to_string()),
        };

        let pubkey = match &args[2] {
            StackItem::ByteString(pk) => pk.as_slice(),
            _ => return Err("checkSig: third argument (pubkey) must be ByteString".to_string()),
        };

        if message.len() > MAX_INPUT_SIZE {
            return Err(format!(
                "checkSig message exceeds maximum size of {} bytes",
                MAX_INPUT_SIZE
            ));
        }

        use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

        let sig = Signature::from_slice(signature)
            .map_err(|_| "checkSig: invalid ECDSA signature format".to_string())?;
        let vk = VerifyingKey::from_sec1_bytes(pubkey)
            .map_err(|_| "checkSig: invalid public key format".to_string())?;

        Ok(StackItem::Boolean(vk.verify(message, &sig).is_ok()))
    }

    /// Compute Murmur3 32-bit hash.
    ///
    /// Arguments: (data: ByteString, seed: Integer)
    /// Returns: ByteString containing the 4-byte hash in little-endian order.
    #[inline]
    fn murmur32(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        if args.is_empty() {
            return Err("murmur32 requires at least 1 argument: data".to_string());
        }

        let data = match &args[0] {
            StackItem::ByteString(d) => d.as_slice(),
            _ => return Err("murmur32: first argument (data) must be ByteString".to_string()),
        };

        if data.len() > MAX_INPUT_SIZE {
            return Err(format!(
                "murmur32 input exceeds maximum size of {} bytes",
                MAX_INPUT_SIZE
            ));
        }

        let seed = args
            .get(1)
            .map(|item| match item {
                StackItem::Integer(n) => Ok(*n as u32),
                _ => Err("murmur32: second argument (seed) must be Integer".to_string()),
            })
            .transpose()?
            .unwrap_or(0);

        let hash = Self::murmur3_32(data, seed);
        Ok(StackItem::ByteString(hash.to_le_bytes().to_vec()))
    }

    /// Murmur3 32-bit hash implementation.
    fn murmur3_32(data: &[u8], seed: u32) -> u32 {
        const C1: u32 = 0xcc9e_2d51;
        const C2: u32 = 0x1b87_3593;

        let mut h = seed;
        let len = data.len();

        // Process 4-byte chunks
        let n_blocks = len / 4;
        for i in 0..n_blocks {
            let offset = i * 4;
            let k = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let k = k.wrapping_mul(C1);
            let k = k.rotate_left(15);
            let k = k.wrapping_mul(C2);
            h ^= k;
            h = h.rotate_left(13);
            h = h.wrapping_mul(5).wrapping_add(0xe654_6b64);
        }

        // Process remaining bytes
        let tail = &data[n_blocks * 4..];
        let mut k1: u32 = 0;
        match tail.len() {
            3 => {
                k1 ^= (tail[2] as u32) << 16;
                k1 ^= (tail[1] as u32) << 8;
                k1 ^= tail[0] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(15);
                k1 = k1.wrapping_mul(C2);
                h ^= k1;
            }
            2 => {
                k1 ^= (tail[1] as u32) << 8;
                k1 ^= tail[0] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(15);
                k1 = k1.wrapping_mul(C2);
                h ^= k1;
            }
            1 => {
                k1 ^= tail[0] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(15);
                k1 = k1.wrapping_mul(C2);
                h ^= k1;
            }
            _ => {}
        }

        // Finalization mix
        h ^= len as u32;
        h ^= h >> 16;
        h = h.wrapping_mul(0x85eb_ca6b);
        h ^= h >> 13;
        h = h.wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        h
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
    ) -> Result<StackItem, NativeContractError> {
        if *hash == self.stdlib.hash() {
            self.stdlib.invoke(method, args)
        } else if *hash == self.cryptolib.hash() {
            self.cryptolib.invoke(method, args)
        } else {
            Err(NativeContractError::Other(format!(
                "Unknown native contract: hash 0x{}",
                hash.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_item::StackItem;

    #[test]
    fn test_sha256_known_output() {
        let crypto = CryptoLib::new();
        // SHA-256 of empty string is well-known
        let result = crypto
            .invoke("sha256", vec![StackItem::ByteString(vec![])])
            .unwrap();
        if let StackItem::ByteString(hash) = result {
            assert_eq!(hash.len(), 32);
            // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
            assert_eq!(hash[0], 0xe3);
            assert_eq!(hash[1], 0xb0);
            assert_eq!(hash[31], 0x55);
        } else {
            panic!("Expected ByteString");
        }
    }

    #[test]
    fn test_base64_encode_decode_roundtrip() {
        let stdlib = StdLib::new();
        let original = b"hello world".to_vec();
        let encoded = stdlib
            .invoke(
                "base64Encode",
                vec![StackItem::ByteString(original.clone())],
            )
            .unwrap();
        let decoded = stdlib.invoke("base64Decode", vec![encoded]).unwrap();
        assert_eq!(decoded, StackItem::ByteString(original));
    }

    #[test]
    fn test_itoa_atoi_roundtrip() {
        let stdlib = StdLib::new();
        let num = 12345i128;
        let as_str = stdlib
            .invoke("itoa", vec![StackItem::Integer(num)])
            .unwrap();
        let back = stdlib.invoke("atoi", vec![as_str]).unwrap();
        assert_eq!(back, StackItem::Integer(num));
    }

    #[test]
    fn test_verify_ecdsa_insufficient_args() {
        let crypto = CryptoLib::new();
        // Only 1 arg - should fail with "requires at least 2 arguments"
        let result = crypto.invoke(
            "verifyWithECDsa",
            vec![StackItem::ByteString(vec![1, 2, 3])],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_ecdsa_missing_pubkey() {
        let crypto = CryptoLib::new();
        // 2 args but no pubkey - should fail with "public key required"
        let result = crypto.invoke(
            "verifyWithECDsa",
            vec![
                StackItem::ByteString(vec![1, 2, 3]),
                StackItem::ByteString(vec![4, 5, 6]),
            ],
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("public key required"));
    }

    #[test]
    fn test_deserialize_oversized_input() {
        let stdlib = StdLib::new();
        // Create input exceeding MAX_INPUT_SIZE (1MB)
        let oversized = vec![0u8; 1024 * 1024 + 1];
        let result = stdlib.invoke("deserialize", vec![StackItem::ByteString(oversized)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds maximum size"));
    }
}
