//! Ethereum-style range / threshold proof for Neo zkVM.
//!
//! **RISC Zero / SP1 analogue:** prove private balance ≥ public minimum
//! without revealing the balance (only a boolean / flag is public).
//!
//! - **Private:** balance (arg0)
//! - **Public:** threshold embedded in script (PUSH8 = 8), result 1/0
//!
//! Script: INITSLOT 0,1; LDARG0; PUSH8; GE; RET
//! (GE leaves boolean on stack)

#[path = "common.rs"]
mod common;

use neo_vm_guest::{OpCode, StackItem};

/// Prove balance >= 8 (threshold hard-coded in script as public constant).
fn range_script_threshold_8() -> Vec<u8> {
    common::script_with_args(
        1,
        &[
            OpCode::LDARG0.byte(),
            OpCode::PUSH8.byte(),
            OpCode::GE.byte(),
            OpCode::RET.byte(),
        ],
    )
}

fn prove_balance_at_least_8(balance: i64) -> neo_zkvm_prover::NeoProof {
    common::prove_mock(
        range_script_threshold_8(),
        vec![StackItem::Integer(balance)],
    )
}

fn main() {
    common::banner("zk_range — private balance ≥ public threshold (Ethereum range style)");

    let rich = 42i64;
    let poor = 3i64;
    let threshold = 8i64;

    println!("Public threshold: {threshold}");
    println!("Private balances: rich={rich}, poor={poor}\n");

    let proof_ok = prove_balance_at_least_8(rich);
    let proof_no = prove_balance_at_least_8(poor);

    let ok_rich = common::verify_mock(&proof_ok);
    let ok_poor = common::verify_mock(&proof_no);

    println!(
        "rich >= {threshold}? verify={} flag={:?}",
        ok_rich,
        common::result_i128(&proof_ok).or_else(|| {
            // Boolean may be StackItem::Boolean
            match proof_ok.output.result.as_ref() {
                Some(StackItem::Boolean(b)) => Some(if *b { 1 } else { 0 }),
                _ => None,
            }
        })
    );
    println!(
        "poor >= {threshold}? verify={} flag={:?}",
        ok_poor,
        match proof_no.output.result.as_ref() {
            Some(StackItem::Boolean(b)) => Some(*b),
            Some(StackItem::Integer(i)) => Some(*i != 0),
            _ => None,
        }
    );

    assert!(ok_rich && ok_poor);
    assert!(matches!(
        proof_ok.output.result.as_ref(),
        Some(StackItem::Boolean(true)) | Some(StackItem::Integer(1))
    ));
    assert!(matches!(
        proof_no.output.result.as_ref(),
        Some(StackItem::Boolean(false)) | Some(StackItem::Integer(0))
    ));
    println!("\nSuccess: verifier sees only pass/fail, not the balance (with real ZK).");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_true(proof: &neo_zkvm_prover::NeoProof) -> bool {
        matches!(
            proof.output.result.as_ref(),
            Some(StackItem::Boolean(true)) | Some(StackItem::Integer(1))
        )
    }

    fn is_false(proof: &neo_zkvm_prover::NeoProof) -> bool {
        matches!(
            proof.output.result.as_ref(),
            Some(StackItem::Boolean(false)) | Some(StackItem::Integer(0))
        )
    }

    #[test]
    fn above_threshold_passes() {
        let p = prove_balance_at_least_8(8);
        assert!(common::verify_mock(&p));
        assert!(is_true(&p));
    }

    #[test]
    fn below_threshold_fails_flag() {
        let p = prove_balance_at_least_8(7);
        assert!(common::verify_mock(&p));
        assert!(is_false(&p));
    }
}
