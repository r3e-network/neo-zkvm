//! Ethereum-style selective disclosure for Neo zkVM.
//!
//! **RISC Zero JSON / "prove one field" analogue:** private record holds multiple
//! fields; prove only that a chosen field equals an expected public value.
//!
//! Encoding (simple packed record as stack args):
//! - arg0 = age (integer)
//! - arg1 = country_code (integer, e.g. 1 = US)
//!
//! We prove **age >= 18** and return country_code as public output so a verifier
//! can check jurisdiction without learning age.
//!
//! Script: INITSLOT 0,2
//!   LDARG0 PUSH16 GE  ASSERT   ; age >= 16 (demo threshold; 18 needs PUSHINT)
//!   LDARG1 RET                 ; disclose country only
//!
//! Uses PUSH16 as age floor for opcode simplicity (demo). Document as analogue.

#[path = "common.rs"]
mod common;

use neo_vm_guest::{OpCode, StackItem};

fn selective_script() -> Vec<u8> {
    // age >= 16 AND return country
    common::script_with_args(
        2,
        &[
            OpCode::LDARG0.byte(),
            OpCode::PUSH16.byte(),
            OpCode::GE.byte(),
            OpCode::ASSERT.byte(), // fault if age < 16
            OpCode::LDARG1.byte(),
            OpCode::RET.byte(),
        ],
    )
}

fn prove_adult_country(age: i64, country: i64) -> neo_zkvm_prover::NeoProof {
    common::prove_mock(
        selective_script(),
        common::neo_call_args(vec![StackItem::Integer(age), StackItem::Integer(country)]),
    )
}

fn main() {
    common::banner("zk_selective_disclosure — prove age gate, reveal only country");

    let age = 25i64; // private
    let country = 1i64; // private until disclosed as public result
    println!("Private record: age={age}, country={country}");
    println!("Public policy: age >= 16 (script constant), disclose country\n");

    let proof = prove_adult_country(age, country);
    let ok = common::verify_mock(&proof);
    let disclosed = common::result_i128(&proof);

    println!("Proof verifies: {ok}");
    println!("Disclosed country code: {disclosed:?}");
    println!("Age is NOT in public output (only country).");

    assert!(ok);
    assert_eq!(disclosed, Some(country as i128));
    println!("\nSuccess: selective disclosure pattern (Mock demo).");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adult_discloses_country() {
        let p = prove_adult_country(30, 44);
        assert!(common::verify_mock(&p));
        assert_eq!(common::result_i128(&p), Some(44));
        assert_eq!(p.output.state, 0);
    }

    #[test]
    fn underage_faults() {
        let p = prove_adult_country(10, 1);
        assert_ne!(p.output.state, 0);
        // Faulted proofs must not pass verification
        assert!(!common::verify_mock(&p));
    }
}
