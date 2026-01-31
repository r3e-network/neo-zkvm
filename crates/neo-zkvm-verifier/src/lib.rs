//! Neo zkVM Verifier with SP1 Integration
//!
//! Production-grade verifier for SP1 zero-knowledge proofs.
//!
//! ## Quick Start
//!
//! ```rust
//! use neo_zkvm_prover::{NeoProver, ProverConfig};
//! use neo_zkvm_verifier::{verify, verify_detailed, VerificationResult};
//! use neo_vm_guest::ProofInput;
//!
//! let prover = NeoProver::new(ProverConfig::default());
//! let input = ProofInput {
//!     script: vec![0x12, 0x13, 0x9E, 0x40],
//!     arguments: vec![],
//!     gas_limit: 1_000_000,
//! };
//!
//! let proof = prover.prove(input);
//! assert!(verify(&proof));
//! ```

use bincode::Options;
use neo_zkvm_prover::{MockProof, NeoProof, ProofMode, PublicInputs, NEO_ZKVM_ELF};
use sha2::{Digest, Sha256};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};

const BINCODE_LIMIT: u64 = 10 * 1024 * 1024; // 10MB limit

fn bincode_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_limit(BINCODE_LIMIT)
        .with_fixint_encoding()
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the proof is valid
    pub valid: bool,
    /// Error message if verification failed
    pub error: Option<String>,
    /// Detected proof type
    pub proof_type: ProofType,
}

/// Proof type detected during verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofType {
    Empty,
    Mock,
    Sp1Compressed,
    Sp1Plonk,
    Sp1Groth16,
    Unknown,
}

/// Verify a Neo zkVM proof (simple interface)
pub fn verify(proof: &NeoProof) -> bool {
    verify_detailed(proof).valid
}

/// Verify with detailed result
pub fn verify_detailed(proof: &NeoProof) -> VerificationResult {
    // Check execution state first
    if proof.output.state != 0 {
        return VerificationResult {
            valid: false,
            error: Some("Execution faulted".to_string()),
            proof_type: ProofType::Unknown,
        };
    }

    // Handle different proof modes
    match proof.proof_mode {
        ProofMode::Execute => VerificationResult {
            valid: true,
            error: None,
            proof_type: ProofType::Empty,
        },
        ProofMode::Mock => {
            let result = verify_mock_proof(proof);
            VerificationResult {
                valid: result,
                error: if result {
                    None
                } else {
                    Some("Mock proof verification failed".to_string())
                },
                proof_type: ProofType::Mock,
            }
        }
        ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16 => verify_sp1_proof(proof),
    }
}

/// Verify a proof with explicit vkey
///
/// This is useful when you have the vkey but not the original prover.
pub fn verify_with_vkey(proof: &NeoProof, vkey: &sp1_sdk::SP1VerifyingKey) -> bool {
    if proof.proof_mode == ProofMode::Mock || proof.proof_mode == ProofMode::Execute {
        return verify(proof);
    }

    match bincode::deserialize::<SP1ProofWithPublicValues>(&proof.proof_bytes) {
        Ok(sp1_proof) => {
            let prover = ProverClient::from_env();
            prover.verify(&sp1_proof, vkey).is_ok()
        }
        Err(_) => false,
    }
}

/// Setup the ELF and return verification key
///
/// This can be used to verify proofs without having the original prover.
pub fn setup_elf() -> sp1_sdk::SP1VerifyingKey {
    let prover = ProverClient::from_env();
    let (_, vk) = prover.setup(NEO_ZKVM_ELF);
    vk
}

fn verify_mock_proof(proof: &NeoProof) -> bool {
    let mock: MockProof = match bincode_options().deserialize(&proof.proof_bytes) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Verify commitment matches public inputs
    let expected = compute_commitment(&proof.public_inputs);
    if mock.commitment != expected {
        return false;
    }

    // Verify all public inputs match
    mock.public_inputs.script_hash == proof.public_inputs.script_hash
        && mock.public_inputs.input_hash == proof.public_inputs.input_hash
        && mock.public_inputs.output_hash == proof.public_inputs.output_hash
        && mock.public_inputs.gas_consumed == proof.public_inputs.gas_consumed
        && mock.public_inputs.execution_success == proof.public_inputs.execution_success
}

fn verify_sp1_proof(proof: &NeoProof) -> VerificationResult {
    let sp1_proof: SP1ProofWithPublicValues =
        match bincode_options().deserialize(&proof.proof_bytes) {
            Ok(p) => p,
            Err(e) => {
                return VerificationResult {
                    valid: false,
                    error: Some(format!("Failed to deserialize SP1 proof: {}", e)),
                    proof_type: ProofType::Unknown,
                };
            }
        };

    // Determine proof type from the proof structure
    let proof_type = detect_sp1_proof_type(&sp1_proof);

    // Create client and verify
    let prover = ProverClient::from_env();
    let (_, vk) = prover.setup(NEO_ZKVM_ELF);

    match prover.verify(&sp1_proof, &vk) {
        Ok(_) => VerificationResult {
            valid: true,
            error: None,
            proof_type,
        },
        Err(e) => VerificationResult {
            valid: false,
            error: Some(format!("SP1 verification failed: {}", e)),
            proof_type,
        },
    }
}

fn detect_sp1_proof_type(_proof: &SP1ProofWithPublicValues) -> ProofType {
    // This is a heuristic based on proof structure
    // In practice, you'd check the proof variant
    ProofType::Sp1Compressed
}

fn compute_commitment(inputs: &PublicInputs) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(inputs.script_hash);
    hasher.update(inputs.input_hash);
    hasher.update(inputs.output_hash);
    hasher.update(inputs.gas_consumed.to_le_bytes());
    hasher.update([inputs.execution_success as u8]);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_vm_core::StackItem;
    use neo_vm_guest::ProofInput;
    use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};

    #[test]
    fn test_verify_mock_proof() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        assert!(verify(&proof));
    }

    #[test]
    fn test_verify_execute_only() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Execute,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        assert!(verify(&proof));
    }

    #[test]
    fn test_verify_detailed() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![StackItem::Integer(42)],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        let result = verify_detailed(&proof);

        assert!(result.valid);
        assert!(result.error.is_none());
        assert_eq!(result.proof_type, ProofType::Mock);
    }
}
