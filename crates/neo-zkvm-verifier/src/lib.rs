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
use neo_zkvm_prover::{
    MockProof, NeoProof, ProofMode, PublicInputs, NEO_ZKVM_ELF, PROOF_FORMAT_VERSION,
};
use sha2::{Digest, Sha256};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1PublicValues};

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
    if proof.proof_format_version != PROOF_FORMAT_VERSION {
        return VerificationResult {
            valid: false,
            error: Some(format!(
                "Unsupported proof format version: expected {} but got {}",
                PROOF_FORMAT_VERSION, proof.proof_format_version
            )),
            proof_type: expected_proof_type(proof.proof_mode),
        };
    }

    if !output_matches_public_inputs(proof) {
        return VerificationResult {
            valid: false,
            error: Some(
                "Output mismatch: NeoProof output fields do not match committed public inputs"
                    .to_string(),
            ),
            proof_type: expected_proof_type(proof.proof_mode),
        };
    }

    match proof.proof_mode {
        ProofMode::Execute => {
            if proof.output.state != 0 {
                return VerificationResult {
                    valid: false,
                    error: Some(format!(
                        "Execution fault: VM exited with state {} (expected 0/Halt)",
                        proof.output.state
                    )),
                    proof_type: ProofType::Empty,
                };
            }
            VerificationResult {
                valid: true,
                error: None,
                proof_type: ProofType::Empty,
            }
        }
        ProofMode::Mock => {
            if proof.output.state != 0 {
                return VerificationResult {
                    valid: false,
                    error: Some(format!(
                        "Execution fault: VM exited with state {} (expected 0/Halt)",
                        proof.output.state
                    )),
                    proof_type: ProofType::Mock,
                };
            }

            let result = verify_mock_proof(proof);
            VerificationResult {
                valid: result,
                error: if result {
                    None
                } else {
                    Some(
                        "Mock proof verification failed: commitment does not match public inputs"
                            .to_string(),
                    )
                },
                proof_type: ProofType::Mock,
            }
        }
        ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16 => {
            let expected_type = match proof.proof_mode {
                ProofMode::Sp1 => ProofType::Sp1Compressed,
                ProofMode::Plonk => ProofType::Sp1Plonk,
                ProofMode::Groth16 => ProofType::Sp1Groth16,
                _ => ProofType::Unknown,
            };
            let result = verify_sp1_proof(proof);
            // Check for proof type mismatch
            if result.valid
                && result.proof_type != expected_type
                && result.proof_type != ProofType::Unknown
            {
                return VerificationResult {
                    valid: false,
                    error: Some(format!(
                        "Proof type mismatch: expected {:?} but got {:?}",
                        expected_type, result.proof_type
                    )),
                    proof_type: result.proof_type,
                };
            }
            result
        }
    }
}

/// Verify a proof with explicit vkey
///
/// This is useful when you have the vkey but not the original prover.
pub fn verify_with_vkey(proof: &NeoProof, vkey: &sp1_sdk::SP1VerifyingKey) -> bool {
    if proof.proof_format_version != PROOF_FORMAT_VERSION {
        return false;
    }

    if !output_matches_public_inputs(proof) {
        return false;
    }

    if proof.proof_mode == ProofMode::Mock || proof.proof_mode == ProofMode::Execute {
        return verify(proof);
    }

    match bincode_options().deserialize::<SP1ProofWithPublicValues>(&proof.proof_bytes) {
        Ok(sp1_proof) => {
            let public_inputs = match decode_public_inputs(&sp1_proof.public_values) {
                Ok(inputs) => inputs,
                Err(_) => return false,
            };
            if !public_inputs_equal(&public_inputs, &proof.public_inputs) {
                return false;
            }
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
    if proof.proof_bytes.is_empty() {
        return VerificationResult {
            valid: false,
            error: Some("SP1 proof bytes are empty".to_string()),
            proof_type: ProofType::Unknown,
        };
    }

    let sp1_proof: SP1ProofWithPublicValues =
        match bincode_options().deserialize(&proof.proof_bytes) {
            Ok(p) => p,
            Err(e) => {
                return VerificationResult {
                    valid: false,
                    error: Some(format!(
                        "Failed to deserialize SP1 proof ({} bytes): {}",
                        proof.proof_bytes.len(),
                        e
                    )),
                    proof_type: ProofType::Unknown,
                };
            }
        };

    // Determine proof type from the proof structure
    let proof_type = detect_sp1_proof_type(&sp1_proof);

    let public_inputs = match decode_public_inputs(&sp1_proof.public_values) {
        Ok(inputs) => inputs,
        Err(e) => {
            return VerificationResult {
                valid: false,
                error: Some(format!(
                    "Failed to decode public inputs from SP1 proof: {}",
                    e
                )),
                proof_type,
            }
        }
    };

    if !public_inputs_equal(&public_inputs, &proof.public_inputs) {
        return VerificationResult {
            valid: false,
            error: Some(
                "Public inputs mismatch: the values committed in the SP1 proof \
                 do not match the claimed public inputs in NeoProof"
                    .to_string(),
            ),
            proof_type,
        };
    }

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
            error: Some(format!(
                "SP1 cryptographic verification failed for {:?} proof: {}",
                proof_type, e
            )),
            proof_type,
        },
    }
}

fn detect_sp1_proof_type(proof: &SP1ProofWithPublicValues) -> ProofType {
    use sp1_sdk::SP1Proof;
    match &proof.proof {
        SP1Proof::Core(_) | SP1Proof::Compressed(_) => ProofType::Sp1Compressed,
        SP1Proof::Plonk(_) => ProofType::Sp1Plonk,
        SP1Proof::Groth16(_) => ProofType::Sp1Groth16,
    }
}

fn decode_public_inputs(values: &SP1PublicValues) -> Result<PublicInputs, String> {
    bincode_options()
        .deserialize(values.as_slice())
        .map_err(|e| format!("Failed to decode public values: {e}"))
}

fn public_inputs_equal(a: &PublicInputs, b: &PublicInputs) -> bool {
    a.script_hash == b.script_hash
        && a.input_hash == b.input_hash
        && a.output_hash == b.output_hash
        && a.gas_consumed == b.gas_consumed
        && a.execution_success == b.execution_success
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

fn hash_proof_output(output: &neo_vm_guest::ProofOutput) -> [u8; 32] {
    let bytes = bincode_options().serialize(output).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

fn output_matches_public_inputs(proof: &NeoProof) -> bool {
    proof.public_inputs.output_hash == hash_proof_output(&proof.output)
        && proof.public_inputs.gas_consumed == proof.output.gas_consumed
        && proof.public_inputs.execution_success == (proof.output.state == 0)
}

fn expected_proof_type(mode: ProofMode) -> ProofType {
    match mode {
        ProofMode::Execute => ProofType::Empty,
        ProofMode::Mock => ProofType::Mock,
        ProofMode::Sp1 => ProofType::Sp1Compressed,
        ProofMode::Plonk => ProofType::Sp1Plonk,
        ProofMode::Groth16 => ProofType::Sp1Groth16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_vm_core::StackItem;
    use neo_vm_guest::ProofInput;
    use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
    use sp1_sdk::SP1PublicValues;

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

    #[test]
    fn test_decode_public_inputs_roundtrip() {
        let inputs = PublicInputs {
            script_hash: [1u8; 32],
            input_hash: [2u8; 32],
            output_hash: [3u8; 32],
            gas_consumed: 42,
            execution_success: true,
        };

        let mut public_values = SP1PublicValues::new();
        public_values.write(&inputs);

        let decoded = decode_public_inputs(&public_values).expect("decode should succeed");
        assert_eq!(decoded.script_hash, inputs.script_hash);
        assert_eq!(decoded.input_hash, inputs.input_hash);
        assert_eq!(decoded.output_hash, inputs.output_hash);
        assert_eq!(decoded.gas_consumed, inputs.gas_consumed);
        assert_eq!(decoded.execution_success, inputs.execution_success);
    }

    #[test]
    fn test_verify_tampered_mock_proof_fails() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };
        let mut proof = prover.prove(input);
        proof.public_inputs.output_hash[0] ^= 0xFF;
        assert!(!verify(&proof));
    }

    #[test]
    fn test_verify_detailed_execute_mode() {
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
        let result = verify_detailed(&proof);
        assert!(result.valid);
        assert_eq!(result.proof_type, ProofType::Empty);
    }

    #[test]
    fn test_verify_faulting_execution_fails() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Execute,
            ..Default::default()
        });
        // Division by zero: PUSH5, PUSH0, DIV, RET
        let input = ProofInput {
            script: vec![0x15, 0x10, 0xA1, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };
        let proof = prover.prove(input);
        let result = verify_detailed(&proof);
        assert!(!result.valid);
    }

    #[test]
    fn test_verify_rejects_tampered_mock_output_result() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let mut proof = prover.prove(input);
        proof.output.result = Some(StackItem::Integer(999));

        assert!(!verify(&proof));
    }

    #[test]
    fn test_verify_rejects_tampered_mock_output_gas() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let mut proof = prover.prove(input);
        proof.output.gas_consumed = proof.output.gas_consumed.saturating_add(1);

        assert!(!verify(&proof));
    }

    #[test]
    fn test_verify_rejects_unknown_proof_format_version() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let mut proof = prover.prove(input);
        proof.proof_format_version = proof.proof_format_version.saturating_add(1);

        let result = verify_detailed(&proof);
        assert!(!result.valid);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Unsupported proof format version"));
    }
}
