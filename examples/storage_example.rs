//! Storage Operations and Merkle Proof Example
//!
//! This example demonstrates how to use Neo zkVM's storage system with
//! Merkle proof support for verifiable state transitions.
//!
//! # Features Demonstrated
//! - Creating and using storage contexts
//! - Basic CRUD operations (Create, Read, Update, Delete)
//! - Tracked storage with change logging
//! - Merkle root computation for state verification
//! - Storage proofs for ZK verification
//!
//! # Use Cases
//! - Smart contract state management
//! - Verifiable key-value storage
//! - State transition proofs

use neo_vm_core::{
    MemoryStorage, StorageBackend, StorageContext, TrackedStorage,
};

fn main() {
    println!("=== Neo zkVM Storage Example ===\n");

    // =========================================================================
    // Part 1: Basic Storage Operations
    // =========================================================================
    println!("--- Part 1: Basic Storage Operations ---\n");

    // Create a storage context for a contract
    // The script_hash identifies which contract owns this storage
    let context = StorageContext {
        script_hash: [0x01; 20], // Example contract hash
        read_only: false,
    };

    // Create in-memory storage backend
    let mut storage = MemoryStorage::new();

    // Store some key-value pairs
    storage.put(&context, b"name", b"Neo zkVM");
    storage.put(&context, b"version", b"1.0.0");
    storage.put(&context, b"counter", &42u64.to_le_bytes());

    println!("Stored values:");
    println!("  name    = {:?}", String::from_utf8_lossy(
        &storage.get(&context, b"name").unwrap()
    ));
    println!("  version = {:?}", String::from_utf8_lossy(
        &storage.get(&context, b"version").unwrap()
    ));

    // Read counter value
    let counter_bytes = storage.get(&context, b"counter").unwrap();
    let counter = u64::from_le_bytes(counter_bytes.try_into().unwrap());
    println!("  counter = {}", counter);

    // =========================================================================
    // Part 2: Merkle Root Computation
    // =========================================================================
    println!("\n--- Part 2: Merkle Root Computation ---\n");

    // Compute Merkle root of current storage state
    let root1 = storage.merkle_root();
    println!("Merkle root (initial): 0x{}", hex_encode(&root1));

    // Modify storage and observe root change
    storage.put(&context, b"counter", &100u64.to_le_bytes());
    let root2 = storage.merkle_root();
    println!("Merkle root (after update): 0x{}", hex_encode(&root2));

    // Roots should be different after modification
    assert_ne!(root1, root2, "Merkle root should change after modification");
    println!("✓ Merkle root changed after state modification");

    // =========================================================================
    // Part 3: Tracked Storage with Change Logging
    // =========================================================================
    println!("\n--- Part 3: Tracked Storage ---\n");

    // TrackedStorage records all changes for proof generation
    let mut tracked = TrackedStorage::new();

    let contract_ctx = StorageContext {
        script_hash: [0x02; 20],
        read_only: false,
    };

    // Perform operations - all changes are logged
    tracked.put(&contract_ctx, b"balance:alice", &1000u64.to_le_bytes());
    tracked.put(&contract_ctx, b"balance:bob", &500u64.to_le_bytes());

    // Simulate a transfer: Alice -> Bob (200 tokens)
    let alice_balance = 1000u64 - 200;
    let bob_balance = 500u64 + 200;
    tracked.put(&contract_ctx, b"balance:alice", &alice_balance.to_le_bytes());
    tracked.put(&contract_ctx, b"balance:bob", &bob_balance.to_le_bytes());

    // Review all changes
    println!("Storage changes recorded:");
    for (i, change) in tracked.changes().iter().enumerate() {
        println!("  Change #{}: key={:?}", i + 1, 
            String::from_utf8_lossy(&change.key));
        if let Some(old) = &change.old_value {
            println!("    old: {} bytes", old.len());
        }
        if let Some(new) = &change.new_value {
            println!("    new: {} bytes", new.len());
        }
    }

    // Get final Merkle root
    let final_root = tracked.merkle_root();
    println!("\nFinal Merkle root: 0x{}", hex_encode(&final_root));

    // =========================================================================
    // Part 4: Prefix Search
    // =========================================================================
    println!("\n--- Part 4: Prefix Search ---\n");

    // Find all balances using prefix search
    let balances = tracked.find(&contract_ctx, b"balance:");
    println!("All balances:");
    for (key, value) in balances {
        let name = String::from_utf8_lossy(&key);
        let amount = u64::from_le_bytes(value.try_into().unwrap());
        println!("  {} = {} tokens", name, amount);
    }

    // =========================================================================
    // Part 5: Read-Only Context
    // =========================================================================
    println!("\n--- Part 5: Read-Only Context ---\n");

    let readonly_ctx = StorageContext {
        script_hash: [0x02; 20],
        read_only: true, // Cannot modify storage
    };

    // Attempt to modify (will be silently ignored)
    let before = tracked.get(&readonly_ctx, b"balance:alice");
    tracked.put(&readonly_ctx, b"balance:alice", &0u64.to_le_bytes());
    let after = tracked.get(&readonly_ctx, b"balance:alice");

    assert_eq!(before, after, "Read-only context should prevent writes");
    println!("✓ Read-only context correctly prevents modifications");

    println!("\n=== Storage Example Complete ===");
}

/// Helper function to encode bytes as hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
