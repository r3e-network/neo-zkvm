//! Deterministic cryptographic helpers for the zk proof guest.
//!
//! These pure functions are the only crypto primitives the proof path
//! is allowed to expose. Host-dependent APIs (CheckSig, storage, time)
//! must not appear here.

use ripemd::{Digest as RipeDigest, Ripemd160};
use sha2::{Digest as ShaDigest, Sha256};

use crate::{StackItem, interop_hash, pop_byte_arg};

/// SHA-256 digest of `data`.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    <Sha256 as ShaDigest>::digest(data).into()
}

/// RIPEMD-160 digest of `data`.
#[must_use]
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
    <Ripemd160 as RipeDigest>::digest(data).into()
}

/// Neo Hash160: `RIPEMD160(SHA256(data))`.
#[must_use]
pub fn hash160(data: &[u8]) -> [u8; 20] {
    ripemd160(&sha256(data))
}

/// Neo Hash256: `SHA256(SHA256(data))`.
#[must_use]
pub fn hash256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}

/// Resolve a known deterministic crypto syscall by interop hash.
///
/// Returns `Ok(true)` if the API was handled, `Ok(false)` if the API is not a
/// known deterministic crypto adapter (caller should reject or route elsewhere).
pub fn try_crypto_syscall(api: u32, stack: &mut Vec<StackItem>) -> Result<bool, String> {
    // Interop hashes are pure functions of the name; compute once per call is
    // cheap relative to hashing payload bytes. Keep this allocation-free.
    if api == interop_hash("System.Crypto.SHA256") {
        let bytes = pop_byte_arg(stack, "System.Crypto.SHA256")?;
        stack.push(StackItem::ByteString(sha256(&bytes).to_vec()));
        return Ok(true);
    }
    if api == interop_hash("System.Crypto.RIPEMD160") {
        let bytes = pop_byte_arg(stack, "System.Crypto.RIPEMD160")?;
        stack.push(StackItem::ByteString(ripemd160(&bytes).to_vec()));
        return Ok(true);
    }
    if api == interop_hash("System.Crypto.Hash160") {
        let bytes = pop_byte_arg(stack, "System.Crypto.Hash160")?;
        stack.push(StackItem::ByteString(hash160(&bytes).to_vec()));
        return Ok(true);
    }
    if api == interop_hash("System.Crypto.Hash256") {
        let bytes = pop_byte_arg(stack, "System.Crypto.Hash256")?;
        stack.push(StackItem::ByteString(hash256(&bytes).to_vec()));
        return Ok(true);
    }
    Ok(false)
}

/// Human-readable name for a known deterministic crypto interop hash, if any.
#[must_use]
pub fn crypto_syscall_name(api: u32) -> Option<&'static str> {
    if api == interop_hash("System.Crypto.SHA256") {
        Some("System.Crypto.SHA256")
    } else if api == interop_hash("System.Crypto.RIPEMD160") {
        Some("System.Crypto.RIPEMD160")
    } else if api == interop_hash("System.Crypto.Hash160") {
        Some("System.Crypto.Hash160")
    } else if api == interop_hash("System.Crypto.Hash256") {
        Some("System.Crypto.Hash256")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
            .collect()
    }

    #[test]
    fn hash_helpers_match_known_vectors() {
        assert_eq!(
            sha256(b"").as_slice(),
            parse_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
        assert_eq!(
            ripemd160(b"").as_slice(),
            parse_hex("9c1185a5c5e9fc54612808977ee8f548b2258d31")
        );
        assert_eq!(hash160(b"abc"), ripemd160(&sha256(b"abc")));
        assert_eq!(hash256(b"abc"), sha256(&sha256(b"abc")));
    }

    #[test]
    fn try_crypto_syscall_handles_sha256() {
        let mut stack = vec![StackItem::ByteString(b"hi".to_vec())];
        let api = interop_hash("System.Crypto.SHA256");
        assert!(try_crypto_syscall(api, &mut stack).expect("ok"));
        match stack.pop() {
            Some(StackItem::ByteString(d)) => assert_eq!(d, sha256(b"hi")),
            other => panic!("unexpected stack: {other:?}"),
        }
    }

    #[test]
    fn try_crypto_syscall_rejects_unknown() {
        let mut stack = vec![];
        assert!(!try_crypto_syscall(0xDEAD_BEEF, &mut stack).expect("ok"));
    }
}
