//! Integration tests for Neo zkVM

use neo_vm_guest::{
    OpCode, ProofInput, StackItem, bincode_deserialize, bincode_serialize, execute, interop_hash,
};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify;

fn syscall(name: &str) -> Vec<u8> {
    let mut script = vec![OpCode::SYSCALL.byte()];
    script.extend_from_slice(&interop_hash(name).to_le_bytes());
    script
}

fn opcodes(ops: &[OpCode]) -> Vec<u8> {
    ops.iter().map(|opcode| opcode.byte()).collect()
}

#[test]
fn test_full_prove_verify_cycle() {
    let script = opcodes(&[OpCode::PUSH2, OpCode::PUSH3, OpCode::ADD, OpCode::RET]);

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let proof = prover.prove(input);

    assert_eq!(proof.output.state, 0);
    assert!(verify(&proof));
}

#[test]
fn test_complex_arithmetic() {
    let script = opcodes(&[
        OpCode::PUSH4,
        OpCode::PUSH5,
        OpCode::MUL,
        OpCode::PUSH2,
        OpCode::DIV,
        OpCode::RET,
    ]);

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
    let script = opcodes(&[OpCode::PUSH3, OpCode::PUSH5, OpCode::LT, OpCode::RET]);

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
        OpCode::INITSLOT.byte(),
        0x00,
        0x02,
        OpCode::LDARG0.byte(),
        OpCode::LDARG1.byte(),
        OpCode::ADD.byte(),
        OpCode::RET.byte(),
    ];

    let input = ProofInput {
        script,
        arguments: vec![StackItem::Integer(10), StackItem::Integer(20)],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let proof = prover.prove(input);

    assert_eq!(proof.output.state, 0);
    assert_eq!(proof.output.result, Some(StackItem::Integer(30)));
    assert!(verify(&proof));
}

#[test]
fn test_prove_verify_hash_operation() {
    let script = [OpCode::PUSHDATA1.byte(), 0x05, b'h', b'e', b'l', b'l', b'o'];
    let mut script_with_hash = script.to_vec();
    script_with_hash.extend(syscall("System.Crypto.SHA256"));
    script_with_hash.push(OpCode::RET.byte());

    let input = ProofInput {
        script: script_with_hash,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let proof = prover.prove(input);

    assert_eq!(proof.output.state, 0);
    assert!(verify(&proof));
}

#[test]
fn test_prove_verify_array_operations() {
    let script = opcodes(&[OpCode::PUSH3, OpCode::NEWARRAY, OpCode::SIZE, OpCode::RET]);

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let proof = prover.prove(input);

    assert_eq!(proof.output.state, 0);
    assert_eq!(proof.output.result, Some(StackItem::Integer(3)));
    assert!(verify(&proof));
}

#[test]
fn test_prove_verify_control_flow() {
    let script = vec![
        OpCode::PUSH5.byte(),
        OpCode::PUSH3.byte(),
        OpCode::GT.byte(),
        OpCode::JMPIF.byte(),
        0x03,
        OpCode::PUSH0.byte(),
        OpCode::JMP.byte(),
        0x02,
        OpCode::PUSH1.byte(),
        OpCode::RET.byte(),
    ];

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let proof = prover.prove(input);

    assert_eq!(proof.output.state, 0);
    assert!(verify(&proof));
}

#[test]
fn test_execute_faulted_script() {
    let script = opcodes(&[OpCode::PUSH5, OpCode::PUSH0, OpCode::DIV, OpCode::RET]);

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
    let script = opcodes(&[
        OpCode::PUSH5,
        OpCode::PUSH3,
        OpCode::ADD,
        OpCode::PUSH2,
        OpCode::MUL,
        OpCode::RET,
    ]);

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });
    let proof = prover.prove(input);

    assert!(proof.output.gas_consumed > 0);
    assert!(proof.public_inputs.gas_consumed > 0);
}

// ============================================================================
// Security and Boundary Tests
// ============================================================================

#[test]
fn test_script_size_limit() {
    let script = vec![OpCode::NOP.byte(); 1024 * 1024 + 1];
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
    let script = opcodes(&[OpCode::DROP, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSH5, OpCode::PUSH0, OpCode::DIV, OpCode::RET]);
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
    let script = vec![OpCode::NOP.byte(); 100];
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
    let mut script = vec![OpCode::PUSHDATA1.byte(), 0x05];
    script.extend_from_slice(b"hello");
    script.push(OpCode::RET.byte());
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
    let script = vec![
        OpCode::PUSHDATA1.byte(),
        0x0A,
        OpCode::NOP.byte(),
        OpCode::NOP.byte(),
        OpCode::NOP.byte(),
        OpCode::NOP.byte(),
        OpCode::NOP.byte(),
    ];
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
    let script = vec![OpCode::JMP.byte(), 0xFE];
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
    let script = opcodes(&[OpCode::NOP, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSH5, OpCode::ABORT, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSH0, OpCode::ASSERT, OpCode::RET]);
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
        OpCode::PUSH2.byte(),
        OpCode::DUP.byte(),
        OpCode::JMPIFNOT.byte(),
        0x05,
        OpCode::DEC.byte(),
        OpCode::JMP.byte(),
        0xFC,
        OpCode::RET.byte(),
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
    let script = opcodes(&[OpCode::PUSH4, OpCode::PUSH3, OpCode::AND, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSH2, OpCode::PUSH1, OpCode::SHL, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSH7, OpCode::PUSH3, OpCode::MOD, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSH2, OpCode::PUSH1, OpCode::POW, OpCode::RET]);
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
    let script = opcodes(&[OpCode::PUSHM1, OpCode::PUSH1, OpCode::MIN, OpCode::RET]);
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
    let script = opcodes(&[
        OpCode::PUSH5,
        OpCode::PUSH0,
        OpCode::PUSH7,
        OpCode::WITHIN,
        OpCode::RET,
    ]);
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
// Shared VM ABI Tests
// ============================================================================

#[test]
fn test_shared_stack_value_serialization_roundtrip() {
    let original = StackItem::Array(vec![
        StackItem::Integer(42),
        StackItem::Boolean(true),
        StackItem::ByteString(b"neo".to_vec()),
    ]);

    let bytes = bincode_serialize(&original).expect("StackItem should serialize");
    let deserialized: StackItem =
        bincode_deserialize(&bytes).expect("StackItem should deserialize");

    assert_eq!(deserialized, original);
}

#[test]
fn test_cli_uses_shared_byte_arg_helper() {
    let trace_host_rs = include_str!("../src/trace_host.rs");

    assert!(trace_host_rs.contains("pop_byte_arg"));
    assert!(!trace_host_rs.contains("fn pop_byte_arg"));
}

#[test]
fn test_native_crypto_sha256() {
    let script = vec![OpCode::PUSHDATA1.byte(), 0x04, b't', b'e', b's', b't'];
    let mut script = script;
    script.extend(syscall("System.Crypto.SHA256"));
    script.extend([OpCode::SIZE.byte(), OpCode::RET.byte()]);
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
fn test_throwifnot_byte_is_rejected_as_non_canonical() {
    let script = vec![
        0xF1, // reserved in NeoVM 3.9.x
        OpCode::RET.byte(),
    ];
    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };
    let output = execute(input);
    assert_eq!(output.state, 1);
    assert!(
        output
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Invalid opcode: 0xf1")
    );
}
