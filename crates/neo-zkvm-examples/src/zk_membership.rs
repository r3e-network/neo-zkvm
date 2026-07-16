//! Ethereum-style set membership / allowlist proof for Neo zkVM.
//!
//! **RISC Zero / SP1 analogue:** prove private value hashes to a public allowlist
//! entry without revealing the value.
//!
//! - **Private:** secret bytes (arg0)
//! - **Public:** expected SHA256 digest (checked by verifier after proof)
//!
//! Script: SYSCALL System.Crypto.SHA256; RET
//! (hash of private ByteString argument)

#[path = "common.rs"]
mod common;

use neo_vm_guest::{OpCode, StackItem, interop_hash, sha256};

fn hash_script() -> Vec<u8> {
    let mut s = vec![OpCode::SYSCALL.byte()];
    s.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());
    s.push(OpCode::RET.byte());
    s
}

fn prove_membership_secret(secret: &[u8]) -> neo_zkvm_prover::NeoProof {
    common::prove_mock(hash_script(), vec![StackItem::ByteString(secret.to_vec())])
}

fn main() {
    common::banner("zk_membership — private secret hashes into public allowlist");

    // Public allowlist of accepted SHA256 digests (e.g. registered commitments).
    let allowlist: Vec<[u8; 32]> = vec![
        sha256(b"alice-member"),
        sha256(b"bob-member"),
        sha256(b"carol-member"),
    ];

    let secret = b"bob-member";
    let proof = prove_membership_secret(secret);
    let ok = common::verify_mock(&proof);
    let out = common::result_bytes(&proof).map(|b| b.to_vec());

    println!("Public allowlist size: {}", allowlist.len());
    println!("Private secret: <redacted>");
    println!("Proof verifies: {ok}");

    let member = out
        .as_ref()
        .map(|d| allowlist.iter().any(|h| h.as_slice() == d.as_slice()))
        .unwrap_or(false);

    println!("Digest in allowlist: {member}");
    if let Some(ref d) = out {
        println!("Public digest: {}", hex::encode(d));
    }

    assert!(ok && member);
    println!("\nSuccess: secret stays private; only membership of its hash is public.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlisted_secret_is_member() {
        let secret = b"alice-member";
        let expected = sha256(secret);
        let proof = prove_membership_secret(secret);
        assert!(common::verify_mock(&proof));
        assert_eq!(common::result_bytes(&proof), Some(expected.as_slice()));
    }

    #[test]
    fn unknown_secret_not_in_allowlist() {
        let allowlist = [sha256(b"alice-member")];
        let proof = prove_membership_secret(b"intruder");
        assert!(common::verify_mock(&proof));
        let dig = common::result_bytes(&proof).unwrap();
        assert!(!allowlist.iter().any(|h| h.as_slice() == dig));
    }
}
