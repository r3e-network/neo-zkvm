//! Neo zkVM SP1 Guest Program - Production Grade
//!
//! Full Neo N3 VM implementation for zero-knowledge proving.
//! Optimized for SP1 with precompile usage where available.

// No main for zkVM - SP1 provides the entrypoint
#![cfg_attr(target_os = "zkvm", no_main)]
#![allow(dead_code)]

#[cfg(target_os = "zkvm")]
sp1_zkvm::entrypoint!(zkvm_main);

#[cfg(any(target_os = "zkvm", test))]
use neo_vm_guest::ProofInput;
use serde::{Deserialize, Serialize};

/// Public values committed to the proof
#[derive(Serialize, Deserialize)]
pub struct PublicValues {
    pub script_hash: [u8; 32],
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub gas_consumed: u64,
    pub execution_success: bool,
}

/// SHA256 hash function
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Main entry point for SP1 zkVM
#[cfg(target_os = "zkvm")]
pub fn zkvm_main() {
    // Read input from host
    let input: ProofInput = sp1_zkvm::io::read();

    // Compute input hash
    let input_bytes = bincode::serialize(&input).unwrap_or_default();
    let input_hash = sha256_hash(&input_bytes);

    // Compute script hash
    let script_hash = sha256_hash(&input.script);

    let output = neo_vm_guest::execute(input);

    // Compute output hash
    let output_bytes = bincode::serialize(&output).unwrap_or_default();
    let output_hash = sha256_hash(&output_bytes);

    // Create public values
    let public_values = PublicValues {
        script_hash: script_hash.into(),
        input_hash: input_hash.into(),
        output_hash,
        gas_consumed: output.gas_consumed,
        execution_success: output.state == 0,
    };

    // Commit public values to the proof
    sp1_zkvm::io::commit(&public_values);
}

/// Main function for non-zkVM targets
#[cfg(not(target_os = "zkvm"))]
fn main() {
    eprintln!("Error: This program must be run in the SP1 zkVM environment.");
    eprintln!("For local testing, use the neo-vm-core crate directly.");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_vm_guest::execute;

    #[test]
    fn test_basic_execution() {
        let output = execute(ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40], // PUSH2 PUSH3 ADD RET
            arguments: vec![],
            gas_limit: 1_000_000,
        });

        assert_eq!(output.state, 0);
        assert_eq!(
            output.result.as_ref().and_then(|item| item.to_integer()),
            Some(5)
        );
    }

    #[test]
    fn test_arithmetic() {
        let output = execute(ProofInput {
            script: vec![0x15, 0x12, 0x9F, 0x40], // PUSH5 PUSH2 SUB RET
            arguments: vec![],
            gas_limit: 1_000_000,
        });

        assert_eq!(output.state, 0);
        assert_eq!(
            output.result.as_ref().and_then(|item| item.to_integer()),
            Some(3)
        );
    }
}
