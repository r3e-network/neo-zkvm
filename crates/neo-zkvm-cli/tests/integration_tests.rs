//! Integration tests for Neo zkVM

use neo_vm_core::StackItem;
use neo_vm_guest::{execute, ProofInput};
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify;

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

// ============================================================================
// End-to-End Proof Generation and Verification Tests
// ============================================================================

#[test]
fn test_prove_verify_with_arguments() {
    let script = vec![
        0x57, 0x00, 0x02, // INITSLOT 0 locals, 2 args
        0x74,             // LDARG0
        0x75,             // LDARG1
        0x9E,             // ADD
        0x40,             // RET
    ];

    let input = ProofInput {
        script,
        arguments: vec![
            StackItem::Integer(10),
            StackItem::Integer(20),
        ],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig::default());
    let proof = prover.prove(input);

    assert_eq!(proof.output.state, 0);
    assert_eq!(proof.output.result, Some(StackItem::Integer(30)));
    assert!(verify(&proof));
}

#[test]
fn test_prove_verify_hash_operation() {
    let script = vec![
        0x0C, 0x05, b'h', b'e', b'l', b'l', b'o',
        0xF0, // SHA256
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
fn test_prove_verify_array_operations() {
    let script = vec![
        0x13, // PUSH3
        0xC3, // NEWARRAY
        0xCA, // SIZE
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
    assert_eq!(proof.output.result, Some(StackItem::Integer(3)));
    assert!(verify(&proof));
}

#[test]
fn test_prove_verify_control_flow() {
    let script = vec![
        0x15, // PUSH5
        0x13, // PUSH3
        0xB7, // GT (5 > 3)
        0x24, 0x03, // JMPIF +3
        0x10, // PUSH0
        0x22, 0x02, // JMP +2
        0x11, // PUSH1
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
fn test_execute_faulted_script() {
    let script = vec![
        0x15, // PUSH5
        0x10, // PUSH0
        0xA1, // DIV (5/0 = fault)
        0x40, // RET
    ];

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let output = execute(input);
    assert_eq!(output.state, 1); // Fault state
}

#[test]
fn test_gas_tracking_in_proof() {
    let script = vec![
        0x15, 0x13, 0x9E, // 5 + 3
        0x12, 0xA0,       // * 2
        0x40,             // RET
    ];

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig::default());
    let proof = prover.prove(input);

    assert!(proof.output.gas_consumed > 0);
    assert!(proof.public_inputs.gas_consumed > 0);
}
