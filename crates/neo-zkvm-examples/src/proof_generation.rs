//! Complete Proof Generation and Verification Example
//!
//! This example demonstrates the full workflow of generating and verifying
//! zero-knowledge proofs for Neo VM script execution.
//!
//! # Workflow
//! 1. Create a Neo VM script
//! 2. Prepare proof input with arguments
//! 3. Generate proof using the prover
//! 4. Verify the proof
//!
//! # Proof Modes
//! - Execute: Run without proof (fastest, for testing)
//! - Mock: Generate mock proof (for development)
//! - SP1: Generate real ZK proof (production)
//! - SP1Plonk: Generate PLONK proof (on-chain verification)

use neo_vm_guest::{OpCode, ProofInput, StackItem};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::{verify, verify_detailed};

fn main() {
    println!("=== Neo zkVM Proof Generation Example ===\n");

    // =========================================================================
    // Part 1: Simple Arithmetic Proof
    // =========================================================================
    println!("--- Part 1: Simple Arithmetic (2 + 3 = 5) ---\n");

    // Create a simple addition script: PUSH2, PUSH3, ADD, RET.
    let add_script = vec![
        OpCode::PUSH2.byte(),
        OpCode::PUSH3.byte(),
        OpCode::ADD.byte(),
        OpCode::RET.byte(),
    ];

    // Prepare proof input
    let input = ProofInput {
        script: add_script.clone(),
        arguments: vec![], // No additional arguments needed
        gas_limit: 100_000,
    };

    // Create prover with mock mode (for demonstration)
    let config = ProverConfig {
        max_cycles: 1_000_000,
        proof_mode: ProofMode::Mock,
        allow_mock_fallback: false,
        deterministic_mock_timestamp: None,
    };
    let prover = NeoProver::new(config);

    // Generate proof
    println!("Generating proof...");
    let proof = prover.prove(input);

    // Display results
    println!("Execution result: {:?}", proof.output.result);
    println!("Gas consumed: {}", proof.output.gas_consumed);
    println!("Proof size: {} bytes", proof.proof_bytes.len());
    println!(
        "Script hash: 0x{}",
        hex::encode(&proof.public_inputs.script_hash[..8])
    );

    // Verify the proof
    let is_valid = verify(&proof);
    println!("Proof valid: {}", is_valid);
    assert!(is_valid, "Proof should be valid");

    // =========================================================================
    // Part 2: Proof with Arguments
    // =========================================================================
    println!("\n--- Part 2: Proof with Stack Arguments ---\n");

    // Script that multiplies two numbers from the stack
    // DUP, MUL, RET (squares the top value)
    let square_script = vec![OpCode::DUP.byte(), OpCode::MUL.byte(), OpCode::RET.byte()];

    let input_with_args = ProofInput {
        script: square_script,
        arguments: vec![StackItem::Integer(7)], // 7² = 49
        gas_limit: 100_000,
    };

    let proof2 = prover.prove(input_with_args);
    println!("Input: 7");
    println!("Result (7²): {:?}", proof2.output.result);
    println!("Verification: {}", verify(&proof2));

    // =========================================================================
    // Part 3: Detailed Verification
    // =========================================================================
    println!("\n--- Part 3: Detailed Verification ---\n");

    let result = verify_detailed(&proof);
    println!("Detailed verification result:");
    println!("  Valid: {}", result.valid);
    if let Some(err) = &result.error {
        println!("  Error: {}", err);
    }

    // =========================================================================
    // Part 4: Execute-Only Mode (No Proof)
    // =========================================================================
    println!("\n--- Part 4: Execute-Only Mode ---\n");

    let exec_config = ProverConfig {
        max_cycles: 1_000_000,
        proof_mode: ProofMode::Execute,
        allow_mock_fallback: false,
        deterministic_mock_timestamp: None,
    };
    let exec_prover = NeoProver::new(exec_config);

    let input3 = ProofInput {
        script: vec![
            OpCode::PUSH5.byte(),
            OpCode::PUSH4.byte(),
            OpCode::MUL.byte(),
            OpCode::RET.byte(),
        ],
        arguments: vec![],
        gas_limit: 100_000,
    };

    let exec_result = exec_prover.prove(input3);
    println!("Execute-only result: {:?}", exec_result.output.result);
    println!(
        "Proof bytes (should be empty): {} bytes",
        exec_result.proof_bytes.len()
    );

    // =========================================================================
    // Part 5: Public Inputs Analysis
    // =========================================================================
    println!("\n--- Part 5: Public Inputs Analysis ---\n");

    println!("Public inputs for verification:");
    println!(
        "  Script hash:       0x{}",
        hex::encode(&proof.public_inputs.script_hash)
    );
    println!(
        "  Input hash:        0x{}",
        hex::encode(&proof.public_inputs.input_hash)
    );
    println!(
        "  Output hash:       0x{}",
        hex::encode(&proof.public_inputs.output_hash)
    );
    println!("  Gas consumed:      {}", proof.public_inputs.gas_consumed);
    println!(
        "  Execution success: {}",
        proof.public_inputs.execution_success
    );

    println!("\n=== Proof Generation Example Complete ===");
}
