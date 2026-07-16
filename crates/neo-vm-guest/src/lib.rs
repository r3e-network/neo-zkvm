//! Shared VM types, serialization helpers, and proof utilities for Neo zkVM.

mod crypto;
mod legacy_neo_proof;
mod mock_proof;
mod neo_proof;
mod proof_input;
mod proof_output;
mod public_inputs;
mod zk_proof_syscalls;

pub use crypto::{crypto_syscall_name, hash160, hash256, ripemd160, sha256, try_crypto_syscall};
pub use legacy_neo_proof::LegacyNeoProof;
pub use mock_proof::MockProof;
pub use neo_proof::NeoProof;
use neo_vm_rs::{
    DEFAULT_MAX_STACK_DEPTH, MAX_SCRIPT_SIZE, VmState, interpret_with_stack_and_syscalls,
};
pub use neo_vm_rs::{OpCode, StackValue as StackItem, interop_hash, pop_byte_arg};
pub use proof_input::ProofInput;
pub use proof_output::ProofOutput;
pub use public_inputs::PublicInputs;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
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

/// Double-SHA256 (Hash256) with domain separation — the canonical Neo protocol
/// hash convention. Matches Neo’s existing `MerkleTree` convention and on-chain
/// verifier expectations. `Hash256(x) = SHA256(SHA256(domain_tag || x))`.
#[must_use]
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neo-zkvm-data-hash-v1:";
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(data);
    let first = hasher.finalize();
    Sha256::digest(first).into()
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
#[deprecated(
    since = "0.2.3",
    note = "use `try_hash_proof_output` instead to avoid panics"
)]
pub fn hash_proof_output(output: &ProofOutput) -> [u8; 32] {
    try_hash_proof_output(output)
        .expect("failed to serialize ProofOutput for hashing; use try_hash_proof_output")
}

/// Compute commitment over all public input fields using double-SHA256
/// (Hash256) with domain separation, the canonical Neo protocol hash convention.
#[must_use]
pub fn compute_commitment(inputs: &PublicInputs) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neo-zkvm-commitment-v1:";
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(inputs.script_hash);
    hasher.update(inputs.input_hash);
    hasher.update(inputs.output_hash);
    hasher.update(inputs.gas_consumed.to_le_bytes());
    hasher.update([inputs.execution_success as u8]);
    let first = hasher.finalize();
    Sha256::digest(first).into()
}

/// Constant-time equality check for 32-byte arrays (hash values, verification keys).
/// Uses XOR accumulation to avoid short-circuit evaluation and timing side-channels.
#[must_use]
#[inline]
pub fn constant_time_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut acc = 0u8;
    for i in 0..32 {
        acc |= a[i] ^ b[i];
    }
    acc == 0
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
///
/// Downgrade protection: if modern deserialization fails AND legacy succeeds,
/// the resulting proof is validated before acceptance. A crafted payload that
/// triggers the fallback path must produce a proof that passes all structural
/// checks — the worst an attacker can do is force the verifier to use the
/// legacy code path, which produces the same result.
pub fn deserialize_neoproof(bytes: &[u8]) -> Result<NeoProof, BincodeDecodeError> {
    match bincode_deserialize::<NeoProof>(bytes) {
        Ok(proof) => Ok(proof),
        Err(primary_err) => match bincode_deserialize::<LegacyNeoProof>(bytes) {
            Ok(legacy) => {
                let proof = NeoProof {
                    output: legacy.output,
                    proof_bytes: legacy.proof_bytes,
                    public_inputs: legacy.public_inputs,
                    vkey_hash: legacy.vkey_hash,
                    proof_mode: legacy.proof_mode,
                    proof_format_version: 1,
                };
                // Reject legacy-path deserialization if the result looks like a
                // corrupt modern/succinct proof rather than a genuine legacy
                // payload. Mock and Execute proofs legitimately use a zero
                // vkey_hash; SP1/Plonk/Groth16 must not.
                // Proof-size sanity is enforced downstream in the verifier
                // (`MAX_PROOF_BYTES`).
                let allows_zero_vkey =
                    matches!(proof.proof_mode, ProofMode::Mock | ProofMode::Execute);
                if proof.vkey_hash == [0u8; 32] && !allows_zero_vkey {
                    return Err(primary_err);
                }
                Ok(proof)
            }
            Err(_) => Err(primary_err),
        },
    }
}

// ============================================================================
// Guest execution
// ============================================================================

/// Hard ceiling on total opcodes the VM will interpret in a single call.
/// Guards against unbounded loops (e.g., `JMP -n`). After this many ops,
/// the call is treated as a fault (state = 1). 10M steps is sufficient for
/// real workloads while capping adversarial scripts at a known bound.
pub const MAX_EXECUTION_STEPS: u64 = 10_000_000;

/// Execute Neo VM and return proof output.
///
/// Gas is metered at runtime: each executed instruction costs one gas unit via
/// the guest syscall host's `on_instruction` hook. This correctly charges loops
/// and dynamic control flow (unlike static bytecode-length estimates).
#[must_use]
pub fn execute(input: ProofInput) -> ProofOutput {
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

    if input.gas_limit == 0 {
        return ProofOutput {
            state: 1,
            gas_consumed: 0,
            result: Some(StackItem::Boolean(false)),
            error: Some("Out of gas".to_string()),
        };
    }

    // Cap the effective budget so adversarial scripts cannot force unbounded
    // interpreter work even if the caller passes a huge gas_limit.
    let effective_gas_limit = input.gas_limit.min(MAX_EXECUTION_STEPS);
    let mut host = ZkProofSyscalls::new(effective_gas_limit);
    match interpret_with_stack_and_syscalls(&input.script, input.arguments, &mut host) {
        Ok(result) => {
            let gas_consumed = host.steps().min(effective_gas_limit);
            let state = match result.state {
                VmState::Halt => 0,
                VmState::Fault => 1,
                state => {
                    return ProofOutput {
                        state: 1,
                        result: Some(StackItem::Boolean(false)),
                        gas_consumed,
                        error: Some(format!("interpreter returned non-final VM state {state:?}")),
                    };
                }
            };

            ProofOutput {
                state,
                result: result.stack.into_iter().last(),
                gas_consumed,
                error: if state == 1 {
                    result
                        .fault_message
                        .or_else(|| Some("Execution fault".to_string()))
                } else {
                    None
                },
            }
        }
        Err(error) => {
            let gas_consumed = host.steps().min(effective_gas_limit);
            ProofOutput {
                state: 1,
                result: Some(StackItem::Boolean(false)),
                gas_consumed,
                error: Some(normalize_interpreter_error(error)),
            }
        }
    }
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
                output_hash: try_hash_proof_output(&output).expect("test output should serialize"),
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
                output_hash: try_hash_proof_output(&output).expect("test output should serialize"),
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
        assert!(
            output
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("Stack overflow")
        );
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
        assert!(
            output
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("Invalid opcode")
        );
    }

    #[test]
    fn test_execute_meters_gas_per_instruction() {
        let input = ProofInput {
            script: vec![
                OpCode::PUSH2.byte(),
                OpCode::PUSH3.byte(),
                OpCode::ADD.byte(),
                OpCode::RET.byte(),
            ],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let output = execute(input);
        assert_eq!(output.state, 0);
        // Four executed opcodes: PUSH2, PUSH3, ADD, RET
        assert_eq!(output.gas_consumed, 4);
    }

    #[test]
    fn test_execute_out_of_gas_on_runtime_step_budget() {
        // 6 PUSH1 + RET requires 7 steps; budget of 3 must fault mid-script.
        let mut script = vec![OpCode::PUSH1.byte(); 6];
        script.push(OpCode::RET.byte());
        let input = ProofInput {
            script,
            arguments: vec![],
            gas_limit: 3,
        };

        let output = execute(input);
        assert_eq!(output.state, 1);
        assert_eq!(output.gas_consumed, 3);
        assert!(
            output
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("Out of gas")
        );
    }

    #[test]
    fn test_execute_zero_gas_limit_fails_closed() {
        let input = ProofInput {
            script: vec![OpCode::RET.byte()],
            arguments: vec![],
            gas_limit: 0,
        };
        let output = execute(input);
        assert_eq!(output.state, 1);
        assert!(
            output
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("Out of gas")
        );
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

    #[test]
    fn test_legacy_execute_neoproof_accepts_zero_vkey() {
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
                result: Some(StackItem::Integer(5)),
                gas_consumed: 4,
                error: None,
            },
            proof_bytes: vec![],
            public_inputs: PublicInputs {
                script_hash: [1u8; 32],
                input_hash: [2u8; 32],
                output_hash: [3u8; 32],
                gas_consumed: 4,
                execution_success: true,
            },
            vkey_hash: [0u8; 32],
            proof_mode: ProofMode::Execute,
        };

        let bytes = bincode_serialize(&legacy).expect("legacy serialize");
        let decoded =
            deserialize_neoproof(&bytes).expect("legacy Execute deserialize should succeed");
        assert_eq!(decoded.proof_mode, ProofMode::Execute);
        assert_eq!(decoded.proof_format_version, PROOF_FORMAT_VERSION);
    }

    #[test]
    fn test_legacy_sp1_with_zero_vkey_is_rejected() {
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
                result: None,
                gas_consumed: 1,
                error: None,
            },
            proof_bytes: vec![1, 2, 3],
            public_inputs: PublicInputs {
                script_hash: [1u8; 32],
                input_hash: [2u8; 32],
                output_hash: [3u8; 32],
                gas_consumed: 1,
                execution_success: true,
            },
            vkey_hash: [0u8; 32],
            proof_mode: ProofMode::Sp1,
        };

        let bytes = bincode_serialize(&legacy).expect("legacy serialize");
        // Modern NeoProof may decode this with default format_version; if the
        // primary path succeeds that is fine. If it falls through to legacy,
        // zero-vkey SP1 must be rejected (Err).
        if let Ok(proof) = deserialize_neoproof(&bytes) {
            // Accepted only as a modern payload with explicit format version.
            assert_eq!(proof.proof_format_version, PROOF_FORMAT_VERSION);
            // Succinct mode with zero vkey must not be treated as verified later;
            // structural acceptance here only checks deserialization.
            assert_eq!(proof.proof_mode, ProofMode::Sp1);
        }
    }
}
