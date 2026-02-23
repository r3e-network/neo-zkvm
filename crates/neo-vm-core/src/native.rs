//! Native Contract Implementations
//!
//! Built-in contracts that provide core blockchain functionality.

use crate::stack_item::StackItem;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Maximum input size for native contract functions (1MB)
const MAX_INPUT_SIZE: usize = 1024 * 1024;

/// Extract a `ByteString` from `args[idx]` with a size check against `MAX_INPUT_SIZE`.
fn extract_bytes<'a>(args: &'a [StackItem], idx: usize, method: &str) -> Result<&'a [u8], String> {
    match args.get(idx) {
        Some(StackItem::ByteString(b)) => {
            if b.len() > MAX_INPUT_SIZE {
                Err(format!(
                    "{method} input exceeds maximum size of {MAX_INPUT_SIZE} bytes"
                ))
            } else {
                Ok(b.as_slice())
            }
        }
        Some(_) => Err(format!("{method} requires ByteString argument")),
        None => Err(format!("{method} missing argument at index {idx}")),
    }
}

/// Contract hash for the Neo N3 StdLib native contract.
pub const STDLIB_HASH: [u8; 20] = [
    0xac, 0xce, 0x6f, 0xd8, 0x0d, 0x44, 0xe1, 0xa3, 0x92, 0x6d, 0xe2, 0x1c, 0xcf, 0x30, 0x96, 0x9a,
    0x22, 0x4b, 0xc0, 0x6b,
];

/// Contract hash for the Neo N3 CryptoLib native contract.
pub const CRYPTOLIB_HASH: [u8; 20] = [
    0x72, 0x6c, 0xb6, 0xe0, 0xcd, 0x8b, 0x0a, 0xc3, 0x3c, 0xe1, 0xde, 0xc0, 0xd4, 0x7e, 0x5c, 0x3c,
    0x4a, 0x6b, 0x8a, 0x0d,
];

/// Errors from native contract invocations.
#[derive(Error, Debug, PartialEq)]
pub enum NativeContractError {
    /// The requested method does not exist on the contract.
    #[error("Unknown method '{method}' on contract {contract}")]
    UnknownMethod {
        /// Name of the target contract.
        contract: &'static str,
        /// Method name that was not found.
        method: String,
    },
    /// An argument failed validation.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    /// Input data exceeds the allowed byte limit.
    #[error("Input exceeds maximum size of {max} bytes")]
    InputTooLarge {
        /// Maximum allowed size in bytes.
        max: usize,
    },
    /// Catch-all for other native contract errors.
    #[error("{0}")]
    Other(String),
}

/// Native contract interface.
pub trait NativeContract {
    /// Return the 20-byte contract hash that identifies this contract.
    fn hash(&self) -> [u8; 20];
    /// Invoke `method` with the given arguments and return the result.
    fn invoke(&self, method: &str, args: Vec<StackItem>) -> Result<StackItem, NativeContractError>;
}

/// StdLib native contract - utility functions
#[derive(Debug, Default)]
pub struct StdLib;

impl StdLib {
    /// Create a new StdLib instance.
    #[inline]
    #[must_use]
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
        let bytes = extract_bytes(&args, 0, "deserialize")?;
        bincode::deserialize(bytes).map_err(|e| format!("deserialize failed: {}", e))
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
        let bytes = extract_bytes(&args, 0, "jsonDeserialize")?;
        let s = String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("jsonDeserialize: invalid UTF-8: {}", e))?;
        serde_json::from_str(&s).map_err(|e| format!("jsonDeserialize: invalid JSON: {}", e))
    }
}

impl StdLib {
    #[inline]
    fn base64_encode(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        let bytes = extract_bytes(&args, 0, "base64Encode")?;
        use base64::{engine::general_purpose::STANDARD, Engine};
        Ok(StackItem::ByteString(STANDARD.encode(bytes).into_bytes()))
    }

    #[inline]
    fn base64_decode(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        let bytes = extract_bytes(&args, 0, "base64Decode")?;
        use base64::{engine::general_purpose::STANDARD, Engine};
        let s = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
        let decoded = STANDARD.decode(s.as_str()).map_err(|e| e.to_string())?;
        Ok(StackItem::ByteString(decoded))
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
        let bytes = extract_bytes(&args, 0, "atoi")?;
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
    }
}

impl NativeContract for StdLib {
    #[inline]
    fn hash(&self) -> [u8; 20] {
        STDLIB_HASH
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
    /// Create a new CryptoLib instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl NativeContract for CryptoLib {
    #[inline]
    fn hash(&self) -> [u8; 20] {
        CRYPTOLIB_HASH
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
        let data = extract_bytes(&args, 0, "sha256")?;
        let hash = Sha256::digest(data);
        Ok(StackItem::ByteString(hash.to_vec()))
    }

    #[inline]
    fn ripemd160(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        let data = extract_bytes(&args, 0, "ripemd160")?;
        use ripemd::Ripemd160;
        let hash = Ripemd160::digest(data);
        Ok(StackItem::ByteString(hash.to_vec()))
    }

    /// Shared ECDSA verification logic.
    fn verify_ecdsa_inner(message: &[u8], signature: &[u8], pubkey: &[u8]) -> Result<bool, String> {
        use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
        let sig = Signature::from_slice(signature)
            .map_err(|_| "Invalid ECDSA signature format".to_string())?;
        let vk = VerifyingKey::from_sec1_bytes(pubkey)
            .map_err(|_| "Invalid public key format".to_string())?;
        Ok(vk.verify(message, &sig).is_ok())
    }

    #[inline]
    fn verify_ecdsa(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        let message = extract_bytes(&args, 0, "verifyWithECDsa")?;
        let signature = extract_bytes(&args, 1, "verifyWithECDsa")?;
        let pubkey = extract_bytes(&args, 2, "verifyWithECDsa")
            .map_err(|_| "verify_ecdsa: public key required".to_string())?;
        Ok(StackItem::Boolean(Self::verify_ecdsa_inner(
            message, signature, pubkey,
        )?))
    }

    /// Verify a single ECDSA signature with a simplified API.
    ///
    /// Arguments: (message: ByteString, signature: ByteString, pubkey: ByteString)
    /// Returns: Boolean indicating whether the signature is valid.
    #[inline]
    fn check_sig(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        let message = extract_bytes(&args, 0, "checkSig")?;
        let signature = extract_bytes(&args, 1, "checkSig")?;
        let pubkey = extract_bytes(&args, 2, "checkSig")?;
        Ok(StackItem::Boolean(Self::verify_ecdsa_inner(
            message, signature, pubkey,
        )?))
    }

    /// Compute Murmur3 32-bit hash.
    ///
    /// Arguments: (data: ByteString, seed: Integer)
    /// Returns: ByteString containing the 4-byte hash in little-endian order.
    #[inline]
    fn murmur32(&self, args: Vec<StackItem>) -> Result<StackItem, String> {
        let data = extract_bytes(&args, 0, "murmur32")?;
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
    /// Create a new registry containing all built-in native contracts.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            stdlib: StdLib::new(),
            cryptolib: CryptoLib::new(),
        }
    }

    /// Dispatch a method call to the native contract identified by `hash`.
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
