//! Ethereum-style "factors" / hello-world ZK example for Neo zkVM.
//!
//! **RISC Zero / SP1 analogue:** prove you know factors of a composite number
//! without revealing the factors.
//!
//! - **Private:** p, q (stack arguments)
//! - **Public claim:** product p*q equals expected n (checked after verify)
//!
//! Script: INITSLOT 0,2; LDARG0; LDARG1; MUL; RET

#[path = "common.rs"]
mod common;

use neo_vm_guest::{OpCode, StackItem};

fn factors_script() -> Vec<u8> {
    common::script_with_args(
        2,
        &[
            OpCode::LDARG0.byte(),
            OpCode::LDARG1.byte(),
            OpCode::MUL.byte(),
            OpCode::RET.byte(),
        ],
    )
}

fn prove_factors(p: i64, q: i64) -> neo_zkvm_prover::NeoProof {
    common::prove_mock(
        factors_script(),
        common::neo_call_args(vec![StackItem::Integer(p), StackItem::Integer(q)]),
    )
}

fn main() {
    common::banner("zk_factors — know p,q of n = p*q (Ethereum hello-world style)");

    let p = 13i64;
    let q = 17i64;
    let n = p * q; // 221 — public claim

    println!("Private factors: p={p}, q={q} (never sent to verifier as plaintext)");
    println!("Public claim:    n={n}\n");

    let proof = prove_factors(p, q);
    let ok = common::verify_mock(&proof);
    let product = common::result_i128(&proof);

    println!("Proof verifies (Mock): {ok}");
    println!("Public output product: {product:?}");
    println!(
        "Verifier accepts claim n={n}: {}",
        ok && product == Some(n as i128)
    );

    assert!(ok);
    assert_eq!(product, Some(n as i128));
    println!("\nSuccess: verifier learns n, not p or q (with real SP1/Groth16).");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_factors_prove_and_match_product() {
        let proof = prove_factors(11, 19);
        assert!(common::verify_mock(&proof));
        assert_eq!(common::result_i128(&proof), Some(209));
    }

    #[test]
    fn wrong_public_claim_is_detectable() {
        let proof = prove_factors(3, 5);
        assert!(common::verify_mock(&proof));
        // Attacker claims n=100 but product is 15
        assert_ne!(common::result_i128(&proof), Some(100));
    }
}
