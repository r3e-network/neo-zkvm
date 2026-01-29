//! Native contract tests for Neo VM Core
//!
//! Tests StdLib and CryptoLib native contracts.

use neo_vm_core::{NativeContract, NativeRegistry, StdLib, CryptoLib, StackItem};

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
    let result = stdlib.invoke("itoa", vec![
        StackItem::Integer(255),
        StackItem::Integer(16),
    ]);
    
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "ff");
    }
}

#[test]
fn test_stdlib_itoa_binary() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("itoa", vec![
        StackItem::Integer(5),
        StackItem::Integer(2),
    ]);
    
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "101");
    }
}

#[test]
fn test_stdlib_atoi_decimal() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("atoi", vec![
        StackItem::ByteString(b"42".to_vec()),
    ]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StackItem::Integer(42));
}

#[test]
fn test_stdlib_atoi_hex() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("atoi", vec![
        StackItem::ByteString(b"ff".to_vec()),
        StackItem::Integer(16),
    ]);
    
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
        let deserialized = stdlib.invoke("deserialize", vec![
            StackItem::ByteString(bytes),
        ]);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), original);
    }
}

#[test]
fn test_stdlib_base64_encode() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("base64Encode", vec![
        StackItem::ByteString(b"hello".to_vec()),
    ]);
    
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(String::from_utf8(bytes).unwrap(), "aGVsbG8=");
    }
}

#[test]
fn test_stdlib_base64_decode() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("base64Decode", vec![
        StackItem::ByteString(b"aGVsbG8=".to_vec()),
    ]);
    
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(bytes)) = result {
        assert_eq!(bytes, b"hello".to_vec());
    }
}

#[test]
fn test_stdlib_json_serialize() {
    let stdlib = StdLib::new();
    let result = stdlib.invoke("jsonSerialize", vec![
        StackItem::Integer(42),
    ]);
    
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
    let result = cryptolib.invoke("sha256", vec![
        StackItem::ByteString(b"hello".to_vec()),
    ]);
    
    assert!(result.is_ok());
    if let Ok(StackItem::ByteString(hash)) = result {
        assert_eq!(hash.len(), 32);
    }
}

#[test]
fn test_cryptolib_sha256_deterministic() {
    let cryptolib = CryptoLib::new();
    let result1 = cryptolib.invoke("sha256", vec![
        StackItem::ByteString(b"test".to_vec()),
    ]);
    let result2 = cryptolib.invoke("sha256", vec![
        StackItem::ByteString(b"test".to_vec()),
    ]);
    
    assert_eq!(result1, result2);
}

#[test]
fn test_cryptolib_ripemd160() {
    let cryptolib = CryptoLib::new();
    let result = cryptolib.invoke("ripemd160", vec![
        StackItem::ByteString(b"hello".to_vec()),
    ]);
    
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
    
    let result = registry.invoke(&stdlib.hash(), "itoa", vec![
        StackItem::Integer(100),
    ]);
    
    assert!(result.is_ok());
}

#[test]
fn test_registry_invoke_cryptolib() {
    let registry = NativeRegistry::new();
    let cryptolib = CryptoLib::new();
    
    let result = registry.invoke(&cryptolib.hash(), "sha256", vec![
        StackItem::ByteString(b"test".to_vec()),
    ]);
    
    assert!(result.is_ok());
}

#[test]
fn test_registry_unknown_contract() {
    let registry = NativeRegistry::new();
    let unknown_hash = [0xFFu8; 20];
    
    let result = registry.invoke(&unknown_hash, "method", vec![]);
    assert!(result.is_err());
}
