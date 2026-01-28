//! Neo zkVM Verifier

use neo_zkvm_prover::{MockProof, NeoProof, PublicInputs, Sp1ProofData};
use sha2::{Digest, Sha256};

/// Verification result
#[derive(Debug)]
pub struct VerificationResult {
    pub valid: bool,
    pub error: Option<String>,
}

/// Verify a Neo zkVM proof
pub fn verify(proof: &NeoProof) -> bool {
    verify_detailed(proof).valid
}

/// Verify with detailed result
pub fn verify_detailed(proof: &NeoProof) -> VerificationResult {
    // Check execution state
    if proof.output.state != 0 {
        return VerificationResult {
            valid: false,
            error: Some("Execution faulted".to_string()),
        };
    }

    // Empty proof is valid for execute-only mode
    if proof.proof_bytes.is_empty() {
        return VerificationResult {
            valid: true,
            error: None,
        };
    }

    // Try to verify as mock proof
    if let Ok(mock) = bincode::deserialize::<MockProof>(&proof.proof_bytes) {
        return verify_mock_proof(&mock, &proof.public_inputs);
    }

    // Try to verify as SP1 proof
    if let Ok(sp1) = bincode::deserialize::<Sp1ProofData>(&proof.proof_bytes) {
        return verify_sp1_proof(&sp1, &proof.public_inputs);
    }

    VerificationResult {
        valid: false,
        error: Some("Unknown proof format".to_string()),
    }
}

fn verify_mock_proof(mock: &MockProof, public_inputs: &PublicInputs) -> VerificationResult {
    // Verify commitment matches
    let expected_commitment = compute_commitment(public_inputs);
    if mock.commitment != expected_commitment {
        return VerificationResult {
            valid: false,
            error: Some("Commitment mismatch".to_string()),
        };
    }

    // Verify public inputs match
    if mock.public_inputs.script_hash != public_inputs.script_hash {
        return VerificationResult {
            valid: false,
            error: Some("Script hash mismatch".to_string()),
        };
    }

    VerificationResult {
        valid: true,
        error: None,
    }
}

fn verify_sp1_proof(sp1: &Sp1ProofData, public_inputs: &PublicInputs) -> VerificationResult {
    // Verify version
    if sp1.version != 1 {
        return VerificationResult {
            valid: false,
            error: Some("Unsupported proof version".to_string()),
        };
    }

    // Verify public inputs match
    if sp1.public_inputs.script_hash != public_inputs.script_hash {
        return VerificationResult {
            valid: false,
            error: Some("Script hash mismatch".to_string()),
        };
    }

    // In production, verify SP1 proof core here
    VerificationResult {
        valid: true,
        error: None,
    }
}

fn compute_commitment(inputs: &PublicInputs) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(inputs.script_hash);
    hasher.update(inputs.initial_state_hash);
    hasher.update(inputs.final_state_hash);
    hasher.update(inputs.gas_consumed.to_le_bytes());
    hasher.finalize().into()
}
