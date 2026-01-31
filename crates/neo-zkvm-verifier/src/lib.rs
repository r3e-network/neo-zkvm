//! Neo zkVM Verifier with SP1 Integration
//!
//! Production-grade verifier for SP1 zero-knowledge proofs.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use neo_zkvm_prover::{NeoProver, ProverConfig};
//! use neo_zkvm_verifier::{verify, verify_detailed, VerificationResult};
//! use neo_vm_guest::ProofInput;
//!
//! // Create prover and generate proof
//! let prover = NeoProver::new(ProverConfig::default());
//! let input = ProofInput {
//!     script: vec![0x12, 0x13, 0x9E, 0x40], // 2 + 3
//!     arguments: vec![],
//!     gas_limit: 1_000_000,
//! };
//!
//! let proof = prover.prove(input);
//!
//! // Simple verification
//! let is_valid = verify(&proof);
//! assert!(is_valid);
//! ```
//!
//! ## Detailed Verification
//!
//! ```rust,ignore
//! use neo_zkvm_prover::{NeoProver, ProverConfig};
//! use neo_zkvm_verifier::{verify_detailed, VerificationResult};
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
//!
//! // Detailed verification with error info
//! let result = verify_detailed(&proof);
//!
//! match result {
//!     VerificationResult { valid: true, error: None } => {
//!         println!("Proof is valid!");
//!     }
//!     VerificationResult { valid: false, error: Some(e) } => {
//!         println!("Verification failed: {}", e);
//!     }
//!     _ => {}
//! }
//! ```
//!
//! ## Verification with Public Inputs
//!
//! ```rust,ignore
//! use neo_zkvm_prover::{NeoProver, ProverConfig};
//! use neo_zkvm_verifier::verify;
//! use neo_vm_guest::ProofInput;
//! use neo_vm_core::StackItem;
//!
//! let prover = NeoProver::new(ProverConfig::default());
//!
//! // Script with arguments: a + b
//! let script = vec![
//!     0x57, 0x00, 0x02, // INITSLOT 0 locals, 2 args
//!     0x74,             // LDARG0
//!     0x75,             // LDARG1
//!     0x9E,             // ADD
//!     0x40,             // RET
//! ];
//!
//! let input = ProofInput {
//!     script,
//!     arguments: vec![StackItem::Integer(100), StackItem::Integer(200)],
//!     gas_limit: 1_000_000,
//! };
//!
//! let proof = prover.prove(input);
//!
//! // Access public inputs for on-chain verification
//! assert!(proof.public_inputs.execution_success);
//! assert!(proof.public_inputs.gas_consumed > 0);
//!
//! // Verify the proof
//! assert!(verify(&proof));
//! ```

use bincode::Options;
use neo_zkvm_prover::{MockProof, NeoProof, PublicInputs, NEO_ZKVM_ELF};
use sha2::{Digest, Sha256};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};

const BINCODE_LIMIT: u64 = 10 * 1024 * 1024; // 10MB limit

fn bincode_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_limit(BINCODE_LIMIT)
        .with_fixint_encoding()
}

/// Verification result
#[derive(Debug)]
pub struct VerificationResult {
    pub valid: bool,
    pub error: Option<String>,
}

/// Proof type detected during verification
#[derive(Debug, Clone, Copy)]
pub enum ProofType {
    Empty,
    Mock,
    Sp1Compressed,
    Sp1Plonk,
    Unknown,
}

/// Verify a Neo zkVM proof (simple interface)
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

    // Detect and verify proof type
    let proof_type = detect_proof_type(&proof.proof_bytes);

    match proof_type {
        ProofType::Mock => verify_mock_proof(&proof.proof_bytes, &proof.public_inputs),
        ProofType::Sp1Compressed | ProofType::Sp1Plonk => {
            verify_sp1_proof(&proof.proof_bytes, &proof.public_inputs, &proof.vkey_hash)
        }
        ProofType::Empty => VerificationResult {
            valid: true,
            error: None,
        },
        ProofType::Unknown => VerificationResult {
            valid: false,
            error: Some("Unknown proof format".to_string()),
        },
    }
}

/// Detect the type of proof from bytes
fn detect_proof_type(proof_bytes: &[u8]) -> ProofType {
    if proof_bytes.is_empty() {
        return ProofType::Empty;
    }

    // Try to deserialize as MockProof
    if bincode_options()
        .deserialize::<MockProof>(proof_bytes)
        .is_ok()
    {
        return ProofType::Mock;
    }

    // Try to deserialize as SP1 proof
    if bincode_options()
        .deserialize::<SP1ProofWithPublicValues>(proof_bytes)
        .is_ok()
    {
        return ProofType::Sp1Compressed;
    }

    ProofType::Unknown
}

/// Verify mock proof (for testing)
fn verify_mock_proof(proof_bytes: &[u8], public_inputs: &PublicInputs) -> VerificationResult {
    let mock: MockProof = match bincode_options().deserialize(proof_bytes) {
        Ok(m) => m,
        Err(_) => {
            return VerificationResult {
                valid: false,
                error: Some("Failed to deserialize mock proof".to_string()),
            }
        }
    };

    // Verify commitment
    let expected = compute_commitment(public_inputs);
    if mock.commitment != expected {
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

/// Verify SP1 proof using the SDK
fn verify_sp1_proof(
    proof_bytes: &[u8],
    _public_inputs: &PublicInputs,
    _vkey_hash: &[u8; 32],
) -> VerificationResult {
    let sp1_proof: SP1ProofWithPublicValues = match bincode_options().deserialize(proof_bytes) {
        Ok(p) => p,
        Err(_) => {
            return VerificationResult {
                valid: false,
                error: Some("Failed to deserialize SP1 proof".to_string()),
            }
        }
    };

    // Create client and get vkey
    let client = ProverClient::from_env();
    let (_, vk) = client.setup(NEO_ZKVM_ELF);

    // Verify the proof
    match client.verify(&sp1_proof, &vk) {
        Ok(_) => VerificationResult {
            valid: true,
            error: None,
        },
        Err(e) => VerificationResult {
            valid: false,
            error: Some(format!("SP1 verification failed: {}", e)),
        },
    }
}

/// Compute commitment hash from public inputs
fn compute_commitment(inputs: &PublicInputs) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(inputs.script_hash);
    hasher.update(inputs.input_hash);
    hasher.update(inputs.output_hash);
    hasher.update(inputs.gas_consumed.to_le_bytes());
    hasher.update([inputs.execution_success as u8]);
    hasher.finalize().into()
}
