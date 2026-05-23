//! Shared VM types, serialization helpers, and proof utilities for Neo zkVM.

mod legacy_neo_proof;
mod mock_proof;
mod neo_proof;
mod proof_input;
mod proof_output;
mod public_inputs;
mod zk_proof_syscalls;

pub use legacy_neo_proof::LegacyNeoProof;
pub use mock_proof::MockProof;
pub use neo_proof::NeoProof;
pub use neo_vm_rs::{interop_hash, pop_byte_arg, OpCode, StackValue as StackItem};
use neo_vm_rs::{
    interpret_with_stack_and_syscalls, VmState, DEFAULT_MAX_STACK_DEPTH, MAX_SCRIPT_SIZE,
};
pub use proof_input::ProofInput;
pub use proof_output::ProofOutput;
pub use public_inputs::PublicInputs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zk_proof_syscalls::ZkProofSyscalls;

/// Bincode size limit (10 MB).
const BINCODE_LIMIT: usize = 10 * 1024 * 1024;

/// Serialization error returned by the workspace bincode codec.
pub type BincodeEncodeError = bincode::error::EncodeError;

/// Deserialization error returned by the workspace bincode codec.
pub type BincodeDecodeError = bincode::error::DecodeError;

/// NeoProof serialization format version.
/// Increment when NeoProof structure or semantics change incompatibly.
pub const PROOF_FORMAT_VERSION: u16 = 1;

/// Maximum script size accepted by zk proof execution.
pub const PROOF_MAX_SCRIPT_SIZE: usize = MAX_SCRIPT_SIZE;

const fn default_proof_format_version() -> u16 {
    PROOF_FORMAT_VERSION
}

/// Configured bincode options shared by prover and verifier.
#[must_use]
pub fn bincode_options() -> impl bincode::config::Config {
    bincode::config::legacy().with_limit::<BINCODE_LIMIT>()
}

/// Serialize with the canonical Neo zkVM bincode configuration.
pub fn bincode_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, BincodeEncodeError> {
    let bytes = bincode::serde::encode_to_vec(value, bincode_options())?;
    if bytes.len() > BINCODE_LIMIT {
        return Err(BincodeEncodeError::OtherString(format!(
            "Encoded bincode payload exceeds limit: {} > {BINCODE_LIMIT}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// Deserialize with the canonical Neo zkVM bincode configuration.
pub fn bincode_deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, BincodeDecodeError> {
    let (value, read) = bincode::serde::decode_from_slice(bytes, bincode_options())?;
    if read == bytes.len() {
        Ok(value)
    } else {
        Err(BincodeDecodeError::OtherString(format!(
            "Trailing bytes after bincode payload: read {read}, total {}",
            bytes.len()
        )))
    }
}

/// Proof mode - determines the type of proof generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofMode {
    /// Execute only, no proof generation.
    Execute,
    /// Mock proof for testing (not cryptographically secure).
    Mock,
    /// SP1 compressed proof.
    Sp1,
    /// SP1 PLONK proof.
    Plonk,
    /// SP1 Groth16 proof.
    Groth16,
}

// ============================================================================
// Shared utility functions
// ============================================================================

/// SHA-256 hash of arbitrary data.
#[must_use]
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// SHA-256 hash of a serialized `ProofOutput`.
pub fn try_hash_proof_output(output: &ProofOutput) -> Result<[u8; 32], BincodeEncodeError> {
    let bytes = bincode_serialize(output)?;
    Ok(hash_data(&bytes))
}

/// SHA-256 hash of a serialized `ProofOutput`.
///
/// This panics if serialization fails; prefer `try_hash_proof_output` in fallible paths.
#[must_use]
pub fn hash_proof_output(output: &ProofOutput) -> [u8; 32] {
    try_hash_proof_output(output)
        .expect("failed to serialize ProofOutput for hashing; use try_hash_proof_output")
}

/// Compute commitment over all public input fields.
#[must_use]
pub fn compute_commitment(inputs: &PublicInputs) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(inputs.script_hash);
    hasher.update(inputs.input_hash);
    hasher.update(inputs.output_hash);
    hasher.update(inputs.gas_consumed.to_le_bytes());
    hasher.update([inputs.execution_success as u8]);
    hasher.finalize().into()
}

/// Check whether two `PublicInputs` are equal.
#[must_use]
pub fn public_inputs_equal(a: &PublicInputs, b: &PublicInputs) -> bool {
    a.script_hash == b.script_hash
        && a.input_hash == b.input_hash
        && a.output_hash == b.output_hash
        && a.gas_consumed == b.gas_consumed
        && a.execution_success == b.execution_success
}

/// Check whether a proof's output is consistent with its public inputs.
#[must_use]
pub fn output_matches_public_inputs(proof: &NeoProof) -> bool {
    let output_hash = match try_hash_proof_output(&proof.output) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    proof.public_inputs.output_hash == output_hash
        && proof.public_inputs.gas_consumed == proof.output.gas_consumed
        && proof.public_inputs.execution_success == (proof.output.state == 0)
}

/// Deserialize a `NeoProof`, with compatibility fallback for legacy proofs that
/// predate `proof_format_version`.
pub fn deserialize_neoproof(bytes: &[u8]) -> Result<NeoProof, BincodeDecodeError> {
    match bincode_deserialize::<NeoProof>(bytes) {
        Ok(proof) => Ok(proof),
        Err(primary_err) => match bincode_deserialize::<LegacyNeoProof>(bytes) {
            Ok(legacy) => Ok(NeoProof {
                output: legacy.output,
                proof_bytes: legacy.proof_bytes,
                public_inputs: legacy.public_inputs,
                vkey_hash: legacy.vkey_hash,
                proof_mode: legacy.proof_mode,
                proof_format_version: 1,
            }),
            Err(_) => Err(primary_err),
        },
    }
}

// ============================================================================
// Guest execution
// ============================================================================

/// Execute Neo VM and return proof output
#[must_use]
pub fn execute(input: ProofInput) -> ProofOutput {
    let estimated_gas = estimate_gas(&input.script, input.arguments.len());
    if input.script.len() > PROOF_MAX_SCRIPT_SIZE {
        return ProofOutput {
            state: 1,
            gas_consumed: 0,
            result: Some(StackItem::Boolean(false)),
            error: Some(format!(
                "Invalid script: script exceeds maximum size of {PROOF_MAX_SCRIPT_SIZE} bytes"
            )),
        };
    }

    if input.arguments.len() > DEFAULT_MAX_STACK_DEPTH {
        return ProofOutput {
            state: 1,
            gas_consumed: 0,
            result: Some(StackItem::Boolean(false)),
            error: Some(format!(
                "Stack overflow: depth {} exceeds limit {}",
                input.arguments.len(),
                DEFAULT_MAX_STACK_DEPTH
            )),
        };
    }

    if estimated_gas > input.gas_limit {
        return ProofOutput {
            state: 1,
            gas_consumed: input.gas_limit,
            result: Some(StackItem::Boolean(false)),
            error: Some("Out of gas".to_string()),
        };
    }

    let mut host = ZkProofSyscalls;
    match interpret_with_stack_and_syscalls(&input.script, input.arguments, &mut host) {
        Ok(result) => {
            let state = match result.state {
                VmState::Halt => 0,
                VmState::Fault => 1,
                state => {
                    return ProofOutput {
                        state: 1,
                        result: Some(StackItem::Boolean(false)),
                        gas_consumed: estimated_gas,
                        error: Some(format!("interpreter returned non-final VM state {state:?}")),
                    };
                }
            };

            ProofOutput {
                state,
                result: result.stack.into_iter().last(),
                gas_consumed: estimated_gas,
                error: if state == 1 {
                    result
                        .fault_message
                        .or_else(|| Some("Execution fault".to_string()))
                } else {
                    None
                },
            }
        }
        Err(error) => ProofOutput {
            state: 1,
            result: Some(StackItem::Boolean(false)),
            gas_consumed: estimated_gas.min(input.gas_limit),
            error: Some(normalize_interpreter_error(error)),
        },
    }
}

#[inline]
fn estimate_gas(script: &[u8], argument_count: usize) -> u64 {
    // `neo-vm-rs` owns canonical execution semantics. zkVM proof metadata still
    // needs a deterministic gas-like counter until the shared interpreter grows
    // full Neo policy gas accounting, so use a monotonic structural estimate.
    (script.len() as u64).saturating_add(argument_count as u64)
}

fn normalize_interpreter_error(error: String) -> String {
    if let Some(hex) = error.strip_prefix("unsupported opcode 0x") {
        return format!("Invalid opcode: 0x{hex}");
    }
    error
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_output_matches_public_inputs_roundtrip() {
        let output = ProofOutput {
            state: 0,
            result: Some(StackItem::Integer(5)),
            gas_consumed: 42,
            error: None,
        };
        let proof = NeoProof {
            output: output.clone(),
            proof_bytes: vec![],
            public_inputs: PublicInputs {
                script_hash: hash_data(b"script"),
                input_hash: hash_data(b"input"),
                output_hash: hash_proof_output(&output),
                gas_consumed: output.gas_consumed,
                execution_success: true,
            },
            vkey_hash: [0u8; 32],
            proof_mode: ProofMode::Mock,
            proof_format_version: PROOF_FORMAT_VERSION,
        };

        assert!(output_matches_public_inputs(&proof));
    }

    #[test]
    fn test_output_mismatch_detected() {
        let output = ProofOutput {
            state: 0,
            result: Some(StackItem::Integer(5)),
            gas_consumed: 42,
            error: None,
        };
        let mut proof = NeoProof {
            output: output.clone(),
            proof_bytes: vec![],
            public_inputs: PublicInputs {
                script_hash: hash_data(b"script"),
                input_hash: hash_data(b"input"),
                output_hash: hash_proof_output(&output),
                gas_consumed: output.gas_consumed,
                execution_success: true,
            },
            vkey_hash: [0u8; 32],
            proof_mode: ProofMode::Mock,
            proof_format_version: PROOF_FORMAT_VERSION,
        };
        proof.output.gas_consumed = proof.output.gas_consumed.saturating_add(1);

        assert!(!output_matches_public_inputs(&proof));
    }

    #[test]
    fn test_compute_commitment_changes_when_inputs_change() {
        let inputs_a = PublicInputs {
            script_hash: [1u8; 32],
            input_hash: [2u8; 32],
            output_hash: [3u8; 32],
            gas_consumed: 10,
            execution_success: true,
        };
        let mut inputs_b = inputs_a.clone();
        inputs_b.gas_consumed = 11;

        assert_ne!(compute_commitment(&inputs_a), compute_commitment(&inputs_b));
    }

    #[test]
    fn test_execute_reports_stack_overflow_for_excessive_arguments() {
        let input = ProofInput {
            script: vec![OpCode::RET.byte()],
            arguments: vec![StackItem::Integer(1); 10_000],
            gas_limit: 1_000_000,
        };

        let output = execute(input);
        assert_eq!(output.state, 1);
        assert!(output
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Stack overflow"));
    }

    #[test]
    fn test_execute_reports_runtime_fault_error() {
        let input = ProofInput {
            script: vec![0xFF], // Invalid opcode
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let output = execute(input);
        assert_eq!(output.state, 1);
        assert!(output
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Invalid opcode"));
    }

    #[test]
    fn test_try_hash_proof_output_rejects_oversized_output() {
        let output = ProofOutput {
            state: 0,
            result: Some(StackItem::ByteString(vec![0u8; 10 * 1024 * 1024 + 1])),
            gas_consumed: 1,
            error: None,
        };
        assert!(try_hash_proof_output(&output).is_err());

        let proof = NeoProof {
            output,
            proof_bytes: vec![],
            public_inputs: PublicInputs {
                script_hash: [0u8; 32],
                input_hash: [0u8; 32],
                output_hash: [0u8; 32],
                gas_consumed: 1,
                execution_success: true,
            },
            vkey_hash: [0u8; 32],
            proof_mode: ProofMode::Mock,
            proof_format_version: PROOF_FORMAT_VERSION,
        };
        assert!(!output_matches_public_inputs(&proof));
    }

    #[test]
    fn test_legacy_neoproof_deserializes_with_default_format_version() {
        #[derive(Serialize)]
        struct LegacyNeoProof {
            output: ProofOutput,
            proof_bytes: Vec<u8>,
            public_inputs: PublicInputs,
            vkey_hash: [u8; 32],
            proof_mode: ProofMode,
        }

        let legacy = LegacyNeoProof {
            output: ProofOutput {
                state: 0,
                result: Some(StackItem::Integer(1)),
                gas_consumed: 1,
                error: None,
            },
            proof_bytes: vec![],
            public_inputs: PublicInputs {
                script_hash: [1u8; 32],
                input_hash: [2u8; 32],
                output_hash: [3u8; 32],
                gas_consumed: 1,
                execution_success: true,
            },
            vkey_hash: [0u8; 32],
            proof_mode: ProofMode::Mock,
        };

        let bytes = bincode_serialize(&legacy).expect("legacy serialize");
        let decoded = deserialize_neoproof(&bytes).expect("legacy deserialize should succeed");

        assert_eq!(decoded.proof_format_version, PROOF_FORMAT_VERSION);
    }
}
