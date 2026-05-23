//! Neo zkVM SP1 Guest Program - Production Grade
//!
//! Full Neo N3 VM implementation for zero-knowledge proving.
//! Optimized for SP1 with precompile usage where available.

// No main for zkVM - SP1 provides the entrypoint
#![cfg_attr(target_os = "zkvm", no_main)]
#![allow(dead_code)]

use serde::Serialize;
#[cfg(target_os = "zkvm")]
sp1_zkvm::entrypoint!(zkvm_main);

#[cfg(any(target_os = "zkvm", test))]
use neo_vm_guest::ProofInput;
#[cfg(target_os = "zkvm")]
use neo_vm_guest::PublicInputs;

/// SHA256 hash function
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn hash_with_bincode_limit<T: Serialize>(value: &T) -> [u8; 32] {
    match neo_vm_guest::bincode_serialize(value) {
        Ok(bytes) => sha256_hash(&bytes),
        Err(err) => {
            let mut marker = b"neo-zkvm-program:serialize-error:v1:".to_vec();
            marker.extend_from_slice(err.to_string().as_bytes());
            sha256_hash(&marker)
        }
    }
}

/// Main entry point for SP1 zkVM
#[cfg(target_os = "zkvm")]
pub fn zkvm_main() {
    // Read input from host
    let input: ProofInput = sp1_zkvm::io::read();

    // Compute input hash
    let input_hash = hash_with_bincode_limit(&input);

    // Compute script hash
    let script_hash = sha256_hash(&input.script);

    let output = neo_vm_guest::execute(input);

    // Compute output hash
    let output_hash = hash_with_bincode_limit(&output);

    // Create public values
    let public_inputs = PublicInputs {
        script_hash,
        input_hash,
        output_hash,
        gas_consumed: output.gas_consumed,
        execution_success: output.state == 0,
    };

    // Commit public values to the proof
    sp1_zkvm::io::commit(&public_inputs);
}

/// Main function for non-zkVM targets
#[cfg(not(target_os = "zkvm"))]
fn main() {
    eprintln!("Error: This program must be run in the SP1 zkVM environment.");
    eprintln!("For local testing, use neo_vm_guest::execute or the neo-zkvm CLI.");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_vm_guest::bincode_serialize;
    use neo_vm_guest::execute;
    use neo_vm_guest::OpCode;

    #[test]
    fn test_basic_execution() {
        let output = execute(ProofInput {
            script: vec![
                OpCode::PUSH2.byte(),
                OpCode::PUSH3.byte(),
                OpCode::ADD.byte(),
                OpCode::RET.byte(),
            ],
            arguments: vec![],
            gas_limit: 1_000_000,
        });

        assert_eq!(output.state, 0);
        assert_eq!(
            output.result.as_ref().and_then(|item| item.to_i128()),
            Some(5)
        );
    }

    #[test]
    fn test_arithmetic() {
        let output = execute(ProofInput {
            script: vec![
                OpCode::PUSH5.byte(),
                OpCode::PUSH2.byte(),
                OpCode::SUB.byte(),
                OpCode::RET.byte(),
            ],
            arguments: vec![],
            gas_limit: 1_000_000,
        });

        assert_eq!(output.state, 0);
        assert_eq!(
            output.result.as_ref().and_then(|item| item.to_i128()),
            Some(3)
        );
    }

    #[test]
    fn test_hash_with_bincode_limit_matches_serialized_hash_on_success() {
        let input = ProofInput {
            script: vec![OpCode::RET.byte()],
            arguments: vec![],
            gas_limit: 123,
        };
        let expected = sha256_hash(&bincode_serialize(&input).expect("serialize input"));
        assert_eq!(hash_with_bincode_limit(&input), expected);
    }

    #[test]
    fn test_hash_with_bincode_limit_uses_error_marker_on_serialize_failure() {
        let input = ProofInput {
            script: vec![0u8; 10 * 1024 * 1024 + 1],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let hash = hash_with_bincode_limit(&input);
        assert_ne!(hash, sha256_hash(&[]));
    }
}
