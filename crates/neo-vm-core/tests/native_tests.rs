//! Native contract tests for Neo VM Core
//!
//! Tests StdLib and CryptoLib native contracts.

use neo_vm_core::{CryptoLib, NativeContract, NativeRegistry, StackItem, StdLib};

// ============================================================================
// StdLib Tests
// ============================================================================

#[test]
fn test_stdlib_hash() {
    let stdlib = StdLib::new();
    let hash = stdlib.hash();
    assert_eq!(hash.len(), 20);
}

#[test]
fn test_stdlib_itoa_decimal() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("itoa", vec![StackItem::Integer(42)]);

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "42");
    }
}

#[test]
fn test_stdlib_itoa_hex() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke(
        "itoa",
        vec![StackItem::Integer(255), StackItem::Integer(16)],
    );

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "ff");
    }
}

#[test]
fn test_stdlib_itoa_binary() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("itoa", vec![StackItem::Integer(5), StackItem::Integer(2)]);

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "101");
    }
}

#[test]
fn test_stdlib_atoi_decimal() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("atoi", vec![StackItem::ByteString(b"42".to_vec())]);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StackItem::Integer(42));
}

#[test]
fn test_stdlib_atoi_hex() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke(
        "atoi",
        vec![
            StackItem::ByteString(b"ff".to_vec()),
            StackItem::Integer(16),
        ],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StackItem::Integer(255));
}

#[test]
fn test_stdlib_serialize_deserialize() {
    let stdlib = StdLib::new();
    let original = StackItem::Integer(12345);

    let serialized = stdlib.invoke("serialize", vec![original.clone()]);
    assert!(serialized.is_ok());

    if let Ok(StackItem::ByteString(bytes)) = serialized {
        let deserialized = stdlib.invoke("deserialize", vec![StackItem::ByteString(bytes)]);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), original);
    }
}

#[test]
fn test_stdlib_base64_encode() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke(
        "base64Encode",
        vec![StackItem::ByteString(b"hello".to_vec())],
    );

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "aGVsbG8=");
    }
}

#[test]
fn test_stdlib_base64_decode() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke(
        "base64Decode",
        vec![StackItem::ByteString(b"aGVsbG8=".to_vec())],
    );

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(bytes, b"hello".to_vec());
    }
}

#[test]
fn test_stdlib_json_serialize() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("jsonSerialize", vec![StackItem::Integer(42)]);

    assert!(result.is_ok());
}

#[test]
fn test_stdlib_unknown_method() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("unknownMethod", vec![]);
    assert!(result.is_err());
}

// ============================================================================
// CryptoLib Tests
// ============================================================================

#[test]
fn test_cryptolib_hash() {
    let cryptolib = CryptoLib::new();
    let hash = cryptolib.hash();
    assert_eq!(hash.len(), 20);
}

#[test]
fn test_cryptolib_sha256() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke("sha256", vec![StackItem::ByteString(b"hello".to_vec())]);

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(hash)) = result {
        assert_eq!(hash.len(), 32);
    }
}

#[test]
fn test_cryptolib_sha256_deterministic() {
    let cryptolib = CryptoLib::new();
    let result1 = cryptolib.invoke("sha256", vec![StackItem::ByteString(b"test".to_vec())]);
    let result2 = cryptolib.invoke("sha256", vec![StackItem::ByteString(b"test".to_vec())]);

    assert_eq!(result1, result2);
}

#[test]
fn test_cryptolib_ripemd160() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke("ripemd160", vec![StackItem::ByteString(b"hello".to_vec())]);

    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(hash)) = result {
        assert_eq!(hash.len(), 20);
    }
}

#[test]
fn test_cryptolib_unknown_method() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke("unknownMethod", vec![]);
    assert!(result.is_err());
}

// ============================================================================
// NativeRegistry Tests
// ============================================================================

#[test]
fn test_registry_invoke_stdlib() {
    let registry = NativeRegistry::new();
    let stdlib = StdLib::new();

    let result = registry.invoke(&stdlib.hash(), "itoa", vec![StackItem::Integer(100)]);

    assert!(result.is_ok());
}

#[test]
fn test_registry_invoke_cryptolib() {
    let registry = NativeRegistry::new();
    let cryptolib = CryptoLib::new();

    let result = registry.invoke(
        &cryptolib.hash(),
        "sha256",
        vec![StackItem::ByteString(b"test".to_vec())],
    );

    assert!(result.is_ok());
}

#[test]
fn test_registry_unknown_contract() {
    let registry = NativeRegistry::new();
    let unknown_hash = [0xFFu8; 20];

    let result = registry.invoke(&unknown_hash, "method", vec![]);
    assert!(result.is_err());
}

// ============================================================================
// Input Size Limit Tests
// ============================================================================

#[test]
fn test_stdlib_serialize_large_input() {
    let stdlib = StdLib::new();
    let large_data = vec![0xFFu8; 1024 * 1024 + 1];
    let result = stdlib.invoke("serialize", vec![StackItem::ByteString(large_data)]);
    assert!(result.is_ok());
}

#[test]
fn test_stdlib_base64_encode_large_input() {
    let stdlib = StdLib::new();
    let large_data = vec![0xFFu8; 1024 * 1024 + 1];
    let result = stdlib.invoke("base64Encode", vec![StackItem::ByteString(large_data)]);
    assert!(result.is_err());
}

#[test]
fn test_stdlib_base64_decode_large_input() {
    let stdlib = StdLib::new();
    let large_data = vec![0x41u8; 1024 * 1024 + 1];
    let result = stdlib.invoke("base64Decode", vec![StackItem::ByteString(large_data)]);
    assert!(result.is_err());
}

#[test]
fn test_stdlib_atoi_large_input() {
    let stdlib = StdLib::new();
    let large_data = vec![0x41u8; 1024 * 1024 + 1];
    let result = stdlib.invoke("atoi", vec![StackItem::ByteString(large_data)]);
    assert!(result.is_err());
}

#[test]
fn test_cryptolib_sha256_large_input() {
    let cryptolib = CryptoLib::new();
    let large_data = vec![0xFFu8; 1024 * 1024 + 1];
    let result = cryptolib.invoke("sha256", vec![StackItem::ByteString(large_data)]);
    assert!(result.is_err());
}

#[test]
fn test_cryptolib_ripemd160_large_input() {
    let cryptolib = CryptoLib::new();
    let large_data = vec![0xFFu8; 1024 * 1024 + 1];
    let result = cryptolib.invoke("ripemd160", vec![StackItem::ByteString(large_data)]);
    assert!(result.is_err());
}

// ============================================================================
// Invalid Input Tests
// ============================================================================

#[test]
fn test_stdlib_itoa_invalid_base() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("itoa", vec![StackItem::Integer(42), StackItem::Integer(8)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Unsupported base"));
    }
}

#[test]
fn test_stdlib_atoi_invalid_base() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke(
        "atoi",
        vec![StackItem::ByteString(b"42".to_vec()), StackItem::Integer(8)],
    );
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Unsupported base"));
    }
}

#[test]
fn test_stdlib_base64_decode_invalid() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke(
        "base64Decode",
        vec![StackItem::ByteString(b"!!!invalid!!!".to_vec())],
    );
    assert!(result.is_err());
}

#[test]
fn test_cryptolib_ecdsa_invalid_signature() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke(
        "verifyWithECDsa",
        vec![
            StackItem::ByteString(b"message".to_vec()),
            StackItem::ByteString(b"invalid-signature".to_vec()),
            StackItem::ByteString(vec![0x04u8; 65]),
        ],
    );
    assert!(result.is_err());
}

#[test]
fn test_cryptolib_ecdsa_invalid_public_key() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke(
        "verifyWithECDsa",
        vec![
            StackItem::ByteString(b"message".to_vec()),
            StackItem::ByteString(vec![0u8; 64]),
            StackItem::ByteString(b"invalid-key".to_vec()),
        ],
    );
    assert!(result.is_err());
}

#[test]
fn test_cryptolib_ecdsa_wrong_args() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke("verifyWithECDsa", vec![StackItem::Integer(42)]);
    assert!(result.is_err());
}

#[test]
fn test_cryptolib_ecdsa_no_public_key() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke(
        "verifyWithECDsa",
        vec![
            StackItem::ByteString(b"message".to_vec()),
            StackItem::ByteString(vec![0u8; 64]),
        ],
    );
    assert!(result.is_err());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_stdlib_itoa_negative() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("itoa", vec![StackItem::Integer(-42)]);
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "-42");
    }
}

#[test]
fn test_stdlib_itoa_zero() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("itoa", vec![StackItem::Integer(0)]);
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "0");
    }
}

#[test]
fn test_stdlib_atoi_negative() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("atoi", vec![StackItem::ByteString(b"-42".to_vec())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StackItem::Integer(-42));
}

#[test]
fn test_stdlib_base64_encode_empty() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("base64Encode", vec![StackItem::ByteString(vec![])]);
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "");
    }
}

#[test]
fn test_stdlib_base64_decode_empty() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("base64Decode", vec![StackItem::ByteString(vec![])]);
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert!(bytes.is_empty());
    }
}

#[test]
fn test_cryptolib_sha256_empty() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke("sha256", vec![StackItem::ByteString(vec![])]);
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(hash)) = result {
        assert_eq!(hash.len(), 32);
        assert_eq!(
            hash,
            vec![
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55
            ]
        );
    }
}
