//! Neo zkVM Verifier with SP1 Integration
//!
//! Production-grade verifier for SP1 zero-knowledge proofs.
//!
//! ## Quick Start
//!
//! ```rust
//! use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
//! use neo_zkvm_verifier::verify;
//! use neo_vm_guest::ProofInput;
//!
//! let prover = NeoProver::new(ProverConfig {
//!     proof_mode: ProofMode::Mock,
//!     ..Default::default()
//! });
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
use neo_vm_guest::{
    bincode_options, compute_commitment, hash_data, output_matches_public_inputs,
    public_inputs_equal, MockProof, NeoProof, ProofMode, PublicInputs, PROOF_FORMAT_VERSION,
};
use neo_zkvm_prover::{NeoProver, NEO_ZKVM_ELF};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1PublicValues};

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub error: Option<String>,
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

/// Verify a proof and require an expected proof mode.
pub fn verify_for_mode(proof: &NeoProof, expected_mode: ProofMode) -> bool {
    verify_detailed_for_mode(proof, expected_mode).valid
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

/// Verify with detailed result and require an expected proof mode.
pub fn verify_detailed_for_mode(proof: &NeoProof, expected_mode: ProofMode) -> VerificationResult {
    if proof.proof_mode != expected_mode {
        return VerificationResult {
            valid: false,
            error: Some(format!(
                "Proof mode mismatch: expected {:?} but got {:?}",
                expected_mode, proof.proof_mode
            )),
            proof_type: expected_proof_type(proof.proof_mode),
        };
    }

    verify_detailed(proof)
}

/// Verify a proof with explicit vkey
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
    if verify_claimed_vkey_hash(&proof.vkey_hash, vkey).is_err() {
        return false;
    }

    match bincode_options().deserialize::<SP1ProofWithPublicValues>(&proof.proof_bytes) {
        Ok(sp1_proof) => {
            let pi = match decode_public_inputs(&sp1_proof.public_values) {
                Ok(inputs) => inputs,
                Err(_) => return false,
            };
            if !public_inputs_equal(&pi, &proof.public_inputs) {
                return false;
            }
            let prover = ProverClient::from_env();
            prover.verify(&sp1_proof, vkey).is_ok()
        }
        Err(_) => false,
    }
}

/// Verify a proof with explicit vkey and require an expected proof mode.
pub fn verify_with_vkey_for_mode(
    proof: &NeoProof,
    expected_mode: ProofMode,
    vkey: &sp1_sdk::SP1VerifyingKey,
) -> bool {
    if proof.proof_mode != expected_mode {
        return false;
    }
    verify_with_vkey(proof, vkey)
}

/// Setup the ELF and return verification key
pub fn setup_elf() -> sp1_sdk::SP1VerifyingKey {
    let prover = ProverClient::from_env();
    let (_, vk) = prover.setup(NEO_ZKVM_ELF);
    vk
}

fn compute_vkey_hash(vkey: &sp1_sdk::SP1VerifyingKey) -> Result<[u8; 32], String> {
    let bytes = bincode_options()
        .serialize(vkey)
        .map_err(|e| format!("Failed to serialize verifying key: {e}"))?;
    Ok(hash_data(&bytes))
}

fn verify_claimed_vkey_hash(
    claimed: &[u8; 32],
    vkey: &sp1_sdk::SP1VerifyingKey,
) -> Result<(), String> {
    let expected = compute_vkey_hash(vkey)?;
    if &expected != claimed {
        return Err(
            "Verifying key hash mismatch: proof was generated with a different key".to_string(),
        );
    }
    Ok(())
}

fn verify_mock_proof(proof: &NeoProof) -> bool {
    let mock: MockProof = match bincode_options().deserialize(&proof.proof_bytes) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let expected = compute_commitment(&proof.public_inputs);
    if mock.commitment != expected {
        return false;
    }

    public_inputs_equal(&mock.public_inputs, &proof.public_inputs)
}

fn verify_sp1_proof(proof: &NeoProof) -> VerificationResult {
    if !NeoProver::is_elf_available() {
        return VerificationResult {
            valid: false,
            error: Some("SP1 verifier ELF is unavailable; cannot verify SP1 proof".to_string()),
            proof_type: expected_proof_type(proof.proof_mode),
        };
    }
    let prover = ProverClient::from_env();
    let (_, vk) = prover.setup(NEO_ZKVM_ELF);
    if let Err(err) = verify_claimed_vkey_hash(&proof.vkey_hash, &vk) {
        return VerificationResult {
            valid: false,
            error: Some(err),
            proof_type: expected_proof_type(proof.proof_mode),
        };
    }

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

    let proof_type = detect_sp1_proof_type(&sp1_proof);

    let pi = match decode_public_inputs(&sp1_proof.public_values) {
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

    if !public_inputs_equal(&pi, &proof.public_inputs) {
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
    use neo_zkvm_prover::{NeoProver, ProverConfig};
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
    fn test_verify_rejects_mismatched_vkey_hash() {
        if cfg!(debug_assertions) || !NeoProver::is_elf_available() {
            return;
        }

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
        proof.proof_mode = ProofMode::Sp1;
        proof.proof_bytes = vec![1, 2, 3];
        proof.vkey_hash = [0xFF; 32];

        let result = verify_detailed(&proof);
        assert!(!result.valid);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Verifying key hash mismatch"));
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

    #[test]
    fn test_verify_for_mode_rejects_mode_mismatch() {
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

        let result = verify_detailed_for_mode(&proof, ProofMode::Sp1);
        assert!(!result.valid);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Proof mode mismatch"));
    }

    #[test]
    fn test_verify_for_mode_accepts_matching_mode() {
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

        assert!(verify_for_mode(&proof, ProofMode::Mock));
    }
}
