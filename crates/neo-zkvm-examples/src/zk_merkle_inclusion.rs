//! Ethereum-style Merkle inclusion sketch for Neo zkVM.
//!
//! **RISC Zero / SP1 analogue:** prove a leaf hashes into a public root using a
//! short path (here: one sibling, left/right fixed for demo).
//!
//! Neo guest exposes Hash256 (double-SHA256). We prove:
//!   root == Hash256( Hash256(leaf) || sibling )   [simplified fixed layout]
//!
//! Actually stack-only Hash256 hashes **one** top ByteString. So we:
//! 1. Hash leaf → h0
//! 2. Concatenate is not a Neo opcode in the guest — so we model a **2-level**
//!    commitment as Hash256(leaf) and check equality to public root in the
//!    **verifier** (path folding done off-chain for the demo), OR hash the
//!    pre-concatenated node bytes as private arg.
//!
//! **Demo model (common for zkVM tutorials):**
//! - Private: `node` = preimage of an interior hash (e.g. sorted pair of children)
//! - Public: `root` expected digest
//! - Script: SYSCALL Hash256; RET
//! - Verifier: digest == root
//!
//! This matches "prove preimage of Merkle node" which is the core of inclusion
//! when the host builds the path and the guest only hashes.

#[path = "common.rs"]
mod common;

use neo_vm_guest::{OpCode, StackItem, hash256, interop_hash};

fn hash256_script() -> Vec<u8> {
    let mut s = vec![OpCode::SYSCALL.byte()];
    s.extend_from_slice(&interop_hash("System.Crypto.Hash256").to_le_bytes());
    s.push(OpCode::RET.byte());
    s
}

/// Host-side path fold (Ethereum-style Merkle, Hash256(left||right)).
fn parent_hash(left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(left.len() + right.len());
    buf.extend_from_slice(left);
    buf.extend_from_slice(right);
    hash256(&buf)
}

fn prove_node_hash(node_bytes: &[u8]) -> neo_zkvm_prover::NeoProof {
    common::prove_mock(
        hash256_script(),
        vec![StackItem::ByteString(node_bytes.to_vec())],
    )
}

fn main() {
    common::banner("zk_merkle_inclusion — prove Merkle node preimage (Ethereum path style)");

    // Toy tree: leaves A, B → root = Hash256(H(A)||H(B)) with H = Hash256
    let leaf_a = b"tx-alice-pays-bob";
    let leaf_b = b"tx-carol-pays-dave";
    let h_a = hash256(leaf_a);
    let h_b = hash256(leaf_b);
    let root = parent_hash(&h_a, &h_b);

    // Prover knows leaf A and sibling h_b; builds parent preimage privately.
    let mut parent_preimage = Vec::new();
    parent_preimage.extend_from_slice(&h_a);
    parent_preimage.extend_from_slice(&h_b);

    println!("Public Merkle root: {}", hex::encode(root));
    println!("Private: leaf A + sibling path (folded into parent preimage)\n");

    let proof = prove_node_hash(&parent_preimage);
    let ok = common::verify_mock(&proof);
    let out = common::result_bytes(&proof).map(|b| b.to_vec());

    println!("Proof verifies: {ok}");
    let matches_root = out.as_ref().map(|d| d.as_slice() == root.as_slice()) == Some(true);
    println!("Public output == root: {matches_root}");

    assert!(ok && matches_root);
    println!("\nSuccess: inclusion reduced to hash preimage of path node (zkVM pattern).");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merkle_parent_preimage_matches_root() {
        let left = hash256(b"L");
        let right = hash256(b"R");
        let root = parent_hash(&left, &right);
        let mut pre = left.to_vec();
        pre.extend_from_slice(&right);
        let proof = prove_node_hash(&pre);
        assert!(common::verify_mock(&proof));
        assert_eq!(common::result_bytes(&proof), Some(root.as_slice()));
    }

    #[test]
    fn wrong_preimage_does_not_match_root() {
        let root = parent_hash(&hash256(b"L"), &hash256(b"R"));
        let proof = prove_node_hash(b"not-the-parent");
        assert!(common::verify_mock(&proof));
        assert_ne!(common::result_bytes(&proof), Some(root.as_slice()));
    }
}
