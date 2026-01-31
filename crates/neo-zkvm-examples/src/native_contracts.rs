//! Native Contracts Example - StdLib and CryptoLib
//!
//! This example demonstrates how to use Neo's built-in native contracts
//! for common operations like serialization, encoding, and cryptography.
//!
//! # Native Contracts Covered
//! - StdLib: Utility functions (serialize, base64, itoa/atoi)
//! - CryptoLib: Cryptographic functions (SHA256, RIPEMD160, ECDSA)
//!
//! # Use Cases
//! - Data serialization for storage
//! - Base64 encoding for external APIs
//! - Cryptographic hashing for verification
//! - Number/string conversions

use neo_vm_core::{CryptoLib, NativeContract, NativeRegistry, StackItem, StdLib};

fn main() {
    println!("=== Neo zkVM Native Contracts Example ===\n");

    // =========================================================================
    // Part 1: StdLib - Serialization
    // =========================================================================
    println!("--- Part 1: StdLib Serialization ---\n");

    let stdlib = StdLib::new();

    // Serialize a complex value
    let data = StackItem::Integer(12345);
    let serialized = stdlib.invoke("serialize", vec![data.clone()]).unwrap();
    println!("Original: {:?}", data);
    if let StackItem::ByteString(bytes) = &serialized {
        println!("Serialized: {} bytes", bytes.len());
    }

    // Deserialize back
    let deserialized = stdlib.invoke("deserialize", vec![serialized]).unwrap();
    println!("Deserialized: {:?}", deserialized);

    // =========================================================================
    // Part 2: StdLib - Base64 Encoding
    // =========================================================================
    println!("\n--- Part 2: Base64 Encoding ---\n");

    let message = StackItem::ByteString(b"Hello, Neo zkVM!".to_vec());
    let encoded = stdlib
        .invoke("base64Encode", vec![message.clone()])
        .unwrap();

    if let StackItem::ByteString(bytes) = &encoded {
        println!("Original: Hello, Neo zkVM!");
        println!("Base64:   {}", String::from_utf8_lossy(bytes));
    }

    // Decode back
    let decoded = stdlib.invoke("base64Decode", vec![encoded]).unwrap();
    if let StackItem::ByteString(bytes) = decoded {
        println!("Decoded:  {}", String::from_utf8_lossy(&bytes));
    }

    // =========================================================================
    // Part 3: StdLib - Number Conversions
    // =========================================================================
    println!("\n--- Part 3: Number Conversions (itoa/atoi) ---\n");

    // Integer to string (various bases)
    let num = StackItem::Integer(255);

    // Decimal
    let dec = stdlib.invoke("itoa", vec![num.clone()]).unwrap();
    if let StackItem::ByteString(b) = &dec {
        println!("255 in decimal: {}", String::from_utf8_lossy(b));
    }

    // Hexadecimal
    let hex = stdlib
        .invoke("itoa", vec![num.clone(), StackItem::Integer(16)])
        .unwrap();
    if let StackItem::ByteString(b) = &hex {
        println!("255 in hex:     {}", String::from_utf8_lossy(b));
    }

    // Binary
    let bin = stdlib
        .invoke("itoa", vec![num.clone(), StackItem::Integer(2)])
        .unwrap();
    if let StackItem::ByteString(b) = &bin {
        println!("255 in binary:  {}", String::from_utf8_lossy(b));
    }

    // String to integer
    let str_num = StackItem::ByteString(b"42".to_vec());
    let parsed = stdlib.invoke("atoi", vec![str_num]).unwrap();
    println!("Parsed '42':    {:?}", parsed);

    // =========================================================================
    // Part 4: CryptoLib - Hashing
    // =========================================================================
    println!("\n--- Part 4: CryptoLib Hashing ---\n");

    let cryptolib = CryptoLib::new();

    let data_to_hash = StackItem::ByteString(b"Neo zkVM".to_vec());

    // SHA256 hash
    let sha256_result = cryptolib
        .invoke("sha256", vec![data_to_hash.clone()])
        .unwrap();
    if let StackItem::ByteString(hash) = &sha256_result {
        println!("SHA256('Neo zkVM'):");
        println!("  {}", hex_encode(hash));
    }

    // RIPEMD160 hash
    let ripemd_result = cryptolib.invoke("ripemd160", vec![data_to_hash]).unwrap();
    if let StackItem::ByteString(hash) = &ripemd_result {
        println!("RIPEMD160('Neo zkVM'):");
        println!("  {}", hex_encode(hash));
    }

    // =========================================================================
    // Part 5: NativeRegistry - Unified Access
    // =========================================================================
    println!("\n--- Part 5: NativeRegistry ---\n");

    let registry = NativeRegistry::new();

    // Get contract hashes
    let stdlib_hash = stdlib.hash();
    let crypto_hash = cryptolib.hash();

    println!("StdLib hash:    0x{}", hex_encode(&stdlib_hash));
    println!("CryptoLib hash: 0x{}", hex_encode(&crypto_hash));

    // Invoke through registry using hash
    let result = registry
        .invoke(&stdlib_hash, "itoa", vec![StackItem::Integer(100)])
        .unwrap();

    if let StackItem::ByteString(b) = result {
        println!(
            "\nRegistry invoke StdLib.itoa(100): {}",
            String::from_utf8_lossy(&b)
        );
    }

    println!("\n=== Native Contracts Example Complete ===");
}

/// Helper function to encode bytes as hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
