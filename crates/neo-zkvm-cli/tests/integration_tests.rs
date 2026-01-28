//! Integration tests for Neo zkVM

use neo_vm_guest::{ProofInput, execute};
use neo_zkvm_prover::{NeoProver, ProverConfig, ProveMode};
use neo_zkvm_verifier::verify;
use neo_vm_core::StackItem;

#[test]
fn test_full_prove_verify_cycle() {
    let script = vec![
        0x12, // PUSH2
        0x13, // PUSH3
        0x9E, // ADD
        0x40, // RET
    ];
    
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    
    let prover = NeoProver::new(ProverConfig::default());
    let proof = prover.prove(input);
    
    assert_eq!(proof.output.state, 0);
    assert!(verify(&proof));
}

#[test]
fn test_complex_arithmetic() {
    let script = vec![
        0x14, // PUSH4
        0x15, // PUSH5
        0xA0, // MUL (4*5=20)
        0x12, // PUSH2
        0xA1, // DIV (20/2=10)
        0x40, // RET
    ];
    
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(10)));
}

#[test]
fn test_comparison_operations() {
    let script = vec![
        0x13, // PUSH3
        0x15, // PUSH5
        0xB5, // LT (3 < 5 = true)
        0x40, // RET
    ];
    
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    
    let output = execute(input);
    assert_eq!(output.result, Some(StackItem::Boolean(true)));
}
