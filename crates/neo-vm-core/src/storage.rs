//! Neo VM Storage Implementation
//!
//! Provides key-value storage for smart contracts with Merkle proof support.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Storage context for a contract
#[derive(Debug, Clone, Default)]
pub struct StorageContext {
    /// Contract script hash (20 bytes)
    pub script_hash: [u8; 20],
    /// Read-only flag
    pub read_only: bool,
}

/// Storage backend trait
pub trait StorageBackend {
    fn get(&self, context: &StorageContext, key: &[u8]) -> Option<Vec<u8>>;
    fn put(&mut self, context: &StorageContext, key: &[u8], value: &[u8]);
    fn delete(&mut self, context: &StorageContext, key: &[u8]);
    fn find(&self, context: &StorageContext, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)>;
}

/// In-memory storage implementation
#[derive(Debug, Clone, Default)]
pub struct MemoryStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    fn make_key(context: &StorageContext, key: &[u8]) -> Vec<u8> {
        let mut full_key = context.script_hash.to_vec();
        full_key.extend_from_slice(key);
        full_key
    }

    /// Compute Merkle root of storage
    pub fn merkle_root(&self) -> [u8; 32] {
        if self.data.is_empty() {
            return [0u8; 32];
        }

        let leaves: Vec<[u8; 32]> = self.data.iter()
            .map(|(k, v)| {
                let mut hasher = Sha256::new();
                hasher.update(k);
                hasher.update(v);
                hasher.finalize().into()
            })
            .collect();

        Self::compute_merkle_root(&leaves)
    }

    fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        if leaves.len() == 1 {
            return leaves[0];
        }

        let mut next_level = Vec::new();
        for chunk in leaves.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(chunk[0]);
            if chunk.len() > 1 {
                hasher.update(chunk[1]);
            } else {
                hasher.update(chunk[0]);
            }
            next_level.push(hasher.finalize().into());
        }

        Self::compute_merkle_root(&next_level)
    }
}

impl StorageBackend for MemoryStorage {
    fn get(&self, context: &StorageContext, key: &[u8]) -> Option<Vec<u8>> {
        let full_key = Self::make_key(context, key);
        self.data.get(&full_key).cloned()
    }

    fn put(&mut self, context: &StorageContext, key: &[u8], value: &[u8]) {
        if context.read_only {
            return;
        }
        let full_key = Self::make_key(context, key);
        self.data.insert(full_key, value.to_vec());
    }

    fn delete(&mut self, context: &StorageContext, key: &[u8]) {
        if context.read_only {
            return;
        }
        let full_key = Self::make_key(context, key);
        self.data.remove(&full_key);
    }

    fn find(&self, context: &StorageContext, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let full_prefix = Self::make_key(context, prefix);
        self.data
            .range(full_prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&full_prefix))
            .map(|(k, v)| {
                let key = k[context.script_hash.len()..].to_vec();
                (key, v.clone())
            })
            .collect()
    }
}

/// Storage proof for ZK verification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageProof {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
    pub merkle_path: Vec<[u8; 32]>,
    pub root: [u8; 32],
}

/// Storage change record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageChange {
    pub script_hash: [u8; 20],
    pub key: Vec<u8>,
    pub old_value: Option<Vec<u8>>,
    pub new_value: Option<Vec<u8>>,
}

/// Tracked storage with change log
#[derive(Debug, Clone, Default)]
pub struct TrackedStorage {
    inner: MemoryStorage,
    changes: Vec<StorageChange>,
}

impl TrackedStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn changes(&self) -> &[StorageChange] {
        &self.changes
    }

    pub fn merkle_root(&self) -> [u8; 32] {
        self.inner.merkle_root()
    }
}

impl StorageBackend for TrackedStorage {
    fn get(&self, context: &StorageContext, key: &[u8]) -> Option<Vec<u8>> {
        self.inner.get(context, key)
    }

    fn put(&mut self, context: &StorageContext, key: &[u8], value: &[u8]) {
        if context.read_only {
            return;
        }
        let old_value = self.inner.get(context, key);
        self.inner.put(context, key, value);
        self.changes.push(StorageChange {
            script_hash: context.script_hash,
            key: key.to_vec(),
            old_value,
            new_value: Some(value.to_vec()),
        });
    }

    fn delete(&mut self, context: &StorageContext, key: &[u8]) {
        if context.read_only {
            return;
        }
        let old_value = self.inner.get(context, key);
        self.inner.delete(context, key);
        self.changes.push(StorageChange {
            script_hash: context.script_hash,
            key: key.to_vec(),
            old_value,
            new_value: None,
        });
    }

    fn find(&self, context: &StorageContext, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.inner.find(context, prefix)
    }
}
