//! Storage integration tests for Neo VM Core
//!
//! Tests storage operations and Merkle proof generation.

use neo_vm_core::{MemoryStorage, StorageBackend, StorageContext, TrackedStorage};

// ============================================================================
// Basic Storage Operations
// ============================================================================

#[test]
fn test_storage_put_get() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key1", b"value1");
    let result = storage.get(&ctx, b"key1");

    assert_eq!(result, Some(b"value1".to_vec()));
}

#[test]
fn test_storage_get_nonexistent() {
    let storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    let result = storage.get(&ctx, b"nonexistent");
    assert_eq!(result, None);
}

#[test]
fn test_storage_delete() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key1", b"value1");
    storage.delete(&ctx, b"key1");

    let result = storage.get(&ctx, b"key1");
    assert_eq!(result, None);
}

#[test]
fn test_storage_overwrite() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key1", b"value1");
    storage.put(&ctx, b"key1", b"value2");

    let result = storage.get(&ctx, b"key1");
    assert_eq!(result, Some(b"value2".to_vec()));
}

// ============================================================================
// Storage Context Isolation
// ============================================================================

#[test]
fn test_storage_context_isolation() {
    let mut storage = MemoryStorage::new();
    let ctx1 = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };
    let ctx2 = StorageContext {
        script_hash: [2u8; 20],
        read_only: false,
    };

    storage.put(&ctx1, b"key", b"value1");
    storage.put(&ctx2, b"key", b"value2");

    assert_eq!(storage.get(&ctx1, b"key"), Some(b"value1".to_vec()));
    assert_eq!(storage.get(&ctx2, b"key"), Some(b"value2".to_vec()));
}

#[test]
fn test_storage_read_only() {
    let mut storage = MemoryStorage::new();
    let ctx_rw = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };
    let ctx_ro = StorageContext {
        script_hash: [1u8; 20],
        read_only: true,
    };

    storage.put(&ctx_rw, b"key", b"value");
    storage.put(&ctx_ro, b"key", b"new_value"); // Should be ignored

    assert_eq!(storage.get(&ctx_rw, b"key"), Some(b"value".to_vec()));
}

#[test]
fn test_storage_read_only_delete() {
    let mut storage = MemoryStorage::new();
    let ctx_rw = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };
    let ctx_ro = StorageContext {
        script_hash: [1u8; 20],
        read_only: true,
    };

    storage.put(&ctx_rw, b"key", b"value");
    storage.delete(&ctx_ro, b"key"); // Should be ignored

    assert_eq!(storage.get(&ctx_rw, b"key"), Some(b"value".to_vec()));
}

// ============================================================================
// Storage Find/Prefix Operations
// ============================================================================

#[test]
fn test_storage_find_prefix() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"user:1", b"alice");
    storage.put(&ctx, b"user:2", b"bob");
    storage.put(&ctx, b"user:3", b"charlie");
    storage.put(&ctx, b"admin:1", b"root");

    let users = storage.find(&ctx, b"user:");
    assert_eq!(users.len(), 3);
}

#[test]
fn test_storage_find_empty_prefix() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key1", b"value1");
    storage.put(&ctx, b"key2", b"value2");

    let all = storage.find(&ctx, b"");
    assert_eq!(all.len(), 2);
}

#[test]
fn test_storage_find_no_match() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key1", b"value1");

    let results = storage.find(&ctx, b"nonexistent:");
    assert_eq!(results.len(), 0);
}

// ============================================================================
// Merkle Root Tests
// ============================================================================

#[test]
fn test_merkle_root_empty() {
    let storage = MemoryStorage::new();
    let root = storage.merkle_root();
    assert_eq!(root, [0u8; 32]);
}

#[test]
fn test_merkle_root_single_item() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value");
    let root = storage.merkle_root();

    assert_ne!(root, [0u8; 32]);
}

#[test]
fn test_merkle_root_changes_on_update() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value1");
    let root1 = storage.merkle_root();

    storage.put(&ctx, b"key", b"value2");
    let root2 = storage.merkle_root();

    assert_ne!(root1, root2);
}

#[test]
fn test_merkle_root_deterministic() {
    let mut storage1 = MemoryStorage::new();
    let mut storage2 = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage1.put(&ctx, b"key1", b"value1");
    storage1.put(&ctx, b"key2", b"value2");

    storage2.put(&ctx, b"key1", b"value1");
    storage2.put(&ctx, b"key2", b"value2");

    assert_eq!(storage1.merkle_root(), storage2.merkle_root());
}

// ============================================================================
// Tracked Storage Tests
// ============================================================================

#[test]
fn test_tracked_storage_records_changes() {
    let mut storage = TrackedStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key1", b"value1");
    storage.put(&ctx, b"key2", b"value2");

    let changes = storage.changes();
    assert_eq!(changes.len(), 2);
}

#[test]
fn test_tracked_storage_records_old_value() {
    let mut storage = TrackedStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value1");
    storage.put(&ctx, b"key", b"value2");

    let changes = storage.changes();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].old_value, None);
    assert_eq!(changes[1].old_value, Some(b"value1".to_vec()));
}

#[test]
fn test_tracked_storage_records_delete() {
    let mut storage = TrackedStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value");
    storage.delete(&ctx, b"key");

    let changes = storage.changes();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[1].new_value, None);
}

#[test]
fn test_tracked_storage_merkle_root() {
    let mut storage = TrackedStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value");
    let root = storage.merkle_root();

    assert_ne!(root, [0u8; 32]);
}

// ============================================================================
// Storage Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_storage_empty_context() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [0u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value");
    let result = storage.get(&ctx, b"key");

    assert_eq!(result, Some(b"value".to_vec()));
}

#[test]
fn test_storage_empty_key() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"", b"value");
    let result = storage.get(&ctx, b"");

    assert_eq!(result, Some(b"value".to_vec()));
}

#[test]
fn test_storage_empty_value() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"");
    let result = storage.get(&ctx, b"key");

    assert_eq!(result, Some(b"".to_vec()));
}

#[test]
fn test_storage_hundred_items() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    // Put 100 key-value pairs
    for i in 0..100 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        storage.put(&ctx, key.as_bytes(), value.as_bytes());
    }

    // Verify all values
    for i in 0..100 {
        let key = format!("key{}", i);
        let expected = format!("value{}", i);
        let result = storage.get(&ctx, key.as_bytes());
        assert_eq!(result, Some(expected.into_bytes()));
    }
}

#[test]
fn test_storage_key_overwrite() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value1");
    storage.put(&ctx, b"key", b"value2");
    storage.put(&ctx, b"key", b"value3");

    let result = storage.get(&ctx, b"key");
    assert_eq!(result, Some(b"value3".to_vec()));
}

#[test]
fn test_merkle_root_empty_storage() {
    let storage = MemoryStorage::new();
    let root = storage.merkle_root();

    assert_eq!(root, [0u8; 32]);
}

#[test]
fn test_merkle_root_single_entry() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    storage.put(&ctx, b"key", b"value");
    let root = storage.merkle_root();

    assert_ne!(root, [0u8; 32]);
    // Same input should produce same root
    let root2 = storage.merkle_root();
    assert_eq!(root, root2);
}

#[test]
fn test_merkle_root_1000_items() {
    let mut storage = MemoryStorage::new();
    let ctx = StorageContext {
        script_hash: [1u8; 20],
        read_only: false,
    };

    // Add 1000 items
    for i in 0..1000 {
        let key = format!("key{:04}", i);
        let value = format!("value{:04}", i);
        storage.put(&ctx, key.as_bytes(), value.as_bytes());
    }

    let root = storage.merkle_root();

    // Root should be non-zero for non-empty storage
    assert_ne!(root, [0u8; 32]);
    // Root should be deterministic
    let root2 = storage.merkle_root();
    assert_eq!(root, root2);
}
