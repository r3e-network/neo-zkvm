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

use neo_vm_core::StackItem;
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProveMode, ProverConfig};
use neo_zkvm_verifier::{verify, verify_detailed};

fn main() {
    println!("=== Neo zkVM Proof Generation Example ===\n");

    // =========================================================================
    // Part 1: Simple Arithmetic Proof
    // =========================================================================
    println!("--- Part 1: Simple Arithmetic (2 + 3 = 5) ---\n");

    // Create a simple addition script: PUSH2, PUSH3, ADD, RET
    // Opcodes: 0x12 = PUSH2, 0x13 = PUSH3, 0x9E = ADD, 0x40 = RET
    let add_script = vec![0x12, 0x13, 0x9E, 0x40];

    // Prepare proof input
    let input = ProofInput {
        script: add_script.clone(),
        arguments: vec![], // No additional arguments needed
        gas_limit: 100_000,
    };

    // Create prover with mock mode (for demonstration)
    let config = ProverConfig {
        max_cycles: 1_000_000,
        prove_mode: ProveMode::Mock,
    };
    let prover = NeoProver::new(config);

    // Generate proof
    println!("Generating proof...");
    let proof = prover.prove(input);

    // Display results
    println!("Execution result: {:?}", proof.output.result);
    println!("Gas consumed: {}", proof.output.gas_consumed);
    println!("Proof size: {} bytes", proof.proof_bytes.len());
    println!("Script hash: 0x{}", hex_encode(&proof.public_inputs.script_hash[..8]));

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
    let square_script = vec![0x4A, 0xA0, 0x40]; // DUP, MUL, RET

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
        prove_mode: ProveMode::Execute,
    };
    let exec_prover = NeoProver::new(exec_config);

    let input3 = ProofInput {
        script: vec![0x15, 0x14, 0xA0, 0x40], // PUSH5, PUSH4, MUL, RET = 20
        arguments: vec![],
        gas_limit: 100_000,
    };

    let exec_result = exec_prover.prove(input3);
    println!("Execute-only result: {:?}", exec_result.output.result);
    println!("Proof bytes (should be empty): {} bytes", exec_result.proof_bytes.len());

    // =========================================================================
    // Part 5: Public Inputs Analysis
    // =========================================================================
    println!("\n--- Part 5: Public Inputs Analysis ---\n");

    println!("Public inputs for verification:");
    println!("  Script hash:       0x{}", hex_encode(&proof.public_inputs.script_hash));
    println!("  Input hash:        0x{}", hex_encode(&proof.public_inputs.input_hash));
    println!("  Output hash:       0x{}", hex_encode(&proof.public_inputs.output_hash));
    println!("  Gas consumed:      {}", proof.public_inputs.gas_consumed);
    println!("  Execution success: {}", proof.public_inputs.execution_success);

    println!("\n=== Proof Generation Example Complete ===");
}

/// Helper function to encode bytes as hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
