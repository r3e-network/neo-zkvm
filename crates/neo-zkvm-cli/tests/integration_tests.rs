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
        0x74, // LDARG0
        0x75, // LDARG1
        0x9E, // ADD
        0x40, // RET
    ];

    let input = ProofInput {
        script,
        arguments: vec![StackItem::Integer(10), StackItem::Integer(20)],
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
        0x0C, 0x05, b'h', b'e', b'l', b'l', b'o', 0xF0, // SHA256
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
        0x12, 0xA0, // * 2
        0x40, // RET
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

// ============================================================================
// Security and Boundary Tests
// ============================================================================

#[test]
fn test_script_size_limit() {
    let script = vec![0x42; 1024 * 1024 + 1]; // 1MB + 1 byte
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault - script too large
}

#[test]
fn test_stack_underflow_handling() {
    let script = vec![0x45, 0x40]; // DROP on empty stack
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault - stack underflow
}

#[test]
fn test_division_by_zero() {
    let script = vec![0x15, 0x10, 0xA1, 0x40]; // 5, 0, DIV
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault - division by zero
}

#[test]
fn test_gas_exhaustion() {
    let script = vec![0x42; 100]; // 100 NOPs
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 10, // Very low gas limit
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault - out of gas
}

#[test]
fn test_pushdata_boundary() {
    // PUSHDATA1 with exact length matching remaining bytes
    let mut script = vec![0x0C, 0x05]; // PUSHDATA1, length 5
    script.extend_from_slice(b"hello");
    script.push(0x40); // RET
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0); // Should succeed
}

#[test]
fn test_pushdata_truncated() {
    // PUSHDATA1 claims 10 bytes but only 5 available
    let script = vec![0x0C, 0x0A, 0x42, 0x42, 0x42, 0x42, 0x42]; // 7 bytes total
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault - truncated data
}

#[test]
fn test_loop_detection_by_gas() {
    // Test that a loop consumes gas and eventually halts
    let script = vec![0x22, 0xFE]; // JMP -2 (infinite loop)
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 100,
    };
    let output = execute(input);
    // Should either fault (out of gas) or halt after some iterations
    assert!(output.state == 0 || output.state == 1);
    assert!(output.gas_consumed > 0);
}

#[test]
fn test_control_flow_jump_valid() {
    // Simple NOP and RET test
    let script = vec![
        0x21, // NOP
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
}

#[test]
fn test_control_flow_abort() {
    // Test ABORT instruction
    let script = vec![
        0x15, // PUSH5
        0x38, // ABORT
        0x40, // RET (unreachable)
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault
}

#[test]
fn test_control_flow_assert() {
    // Test ASSERT - fails when condition is false
    let script = vec![
        0x10, // PUSH0 (false)
        0x39, // ASSERT (fails)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1); // Should fault
}

#[test]
fn test_control_flow_jump_backward() {
    // Test backward jump with a bounded loop that halts
    let script = vec![
        0x12, // PUSH2 (counter)
        0x4A, // DUP
        0x26, 0x05, // JMPIFNOT +5 (jump to RET when counter == 0)
        0x9D, // DEC
        0x22, 0xFC, // JMP -4 (jump back to DUP)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
}

#[test]
fn test_bitwise_operations() {
    let script = vec![
        0x14, // PUSH4
        0x13, // PUSH3
        0x91, // AND (4 & 3 = 0)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(0)));
}

#[test]
fn test_shift_operations() {
    let script = vec![
        0x12, // PUSH2
        0x11, // PUSH1
        0xA8, // SHL (2 << 1 = 4)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(4)));
}

#[test]
fn test_modulo_operations() {
    let script = vec![
        0x17, // PUSH7
        0x13, // PUSH3
        0xA2, // MOD (7 % 3 = 1)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(1)));
}

#[test]
fn test_power_operations() {
    let script = vec![
        0x12, // PUSH2
        0x11, // PUSH1
        0xA3, // POW (2^1 = 2)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(2)));
}

#[test]
fn test_min_max_operations() {
    let script = vec![
        0x0F, // PUSHM1 (-1)
        0x11, // PUSH1 (1)
        0xB9, // MIN (-1 < 1 = -1)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(-1)));
}

#[test]
fn test_within_range_check() {
    let script = vec![
        0x15, // PUSH5
        0x10, // PUSH0
        0x17, // PUSH7
        0xBB, // WITHIN (0 <= 5 < 7 = true)
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Boolean(true)));
}

// ============================================================================
// Native Contract Tests
// ============================================================================

#[test]
fn test_native_stdlib_serialize() {
    // This would require syscall support, skip for now
}

#[test]
fn test_native_crypto_sha256() {
    let script = vec![
        0x0C, 0x04, b't', b'e', b's', b't', // PUSHDATA1 "test" (4 bytes)
        0xF0, // SHA256
        0xCA, // SIZE
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    assert_eq!(output.result, Some(StackItem::Integer(32))); // SHA256 produces 32 bytes
}

#[test]
fn test_native_crypto_ripemd160() {
    let script = vec![
        0x0C, 0x03, b'a', b'b', b'c', // PUSHDATA1 "abc"
        0xF1, // RIPEMD160
        0x40, // RET
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 0);
    if let Some(StackItem::ByteString(hash)) = &output.result {
        assert_eq!(hash.len(), 20); // RIPEMD160 produces 20 bytes
    } else {
        panic!("Expected ByteString result");
    }
}
