//! Neo zkVM Prover with SP1 Integration
//!
//! Production-grade prover using SP1 zkVM for generating zero-knowledge proofs.
//!
//! ## Quick Start
//!
//! ```rust
//! use neo_zkvm_prover::{NeoProver, ProverConfig, ProofMode};
//! use neo_vm_guest::ProofInput;
//!
//! // Use mock mode for local/debug development.
//! let prover = NeoProver::new(ProverConfig {
//!     proof_mode: ProofMode::Mock,
//!     ..Default::default()
//! });
//!
//! // Define input
//! let input = ProofInput {
//!     script: vec![0x12, 0x13, 0x9E, 0x40], // 2 + 3
//!     arguments: vec![],
//!     gas_limit: 1_000_000,
//! };
//!
//! // Generate proof
//! let proof = prover.prove(input);
//! ```

use bincode::Options;
mod elf_markers;

use neo_vm_guest::{
    bincode_options, compute_commitment, execute, hash_data, output_matches_public_inputs,
    public_inputs_equal, try_hash_proof_output, ProofInput, ProofOutput,
};
use sp1_sdk::{ProverClient, SP1ProofMode, SP1PublicValues, SP1Stdin};

// Re-export shared types so downstream crates (CLI, examples) keep compiling.
pub use neo_vm_guest::{MockProof, NeoProof, ProofMode, PublicInputs, PROOF_FORMAT_VERSION};

/// SP1 ELF binary - embedded at compile time.
pub const NEO_ZKVM_ELF: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/elf/riscv32im-succinct-zkvm-elf"));

type DynError = Box<dyn std::error::Error>;
type Sp1ProofArtifacts = (Vec<u8>, [u8; 32], PublicInputs);
type Sp1FallbackResult = (Vec<u8>, [u8; 32], ProofMode, Option<PublicInputs>);

/// Prover configuration
#[derive(Clone, Debug)]
pub struct ProverConfig {
    /// Maximum cycles for SP1 execution
    pub max_cycles: u64,
    /// Proof mode (determines proof type and verification cost)
    pub proof_mode: ProofMode,
    /// Allow cryptographic proof modes to fall back to mock proofs on failure.
    ///
    /// Defaults to `false` so production callers fail closed unless they opt in.
    pub allow_mock_fallback: bool,
    /// Optional fixed timestamp for deterministic mock proofs.
    pub deterministic_mock_timestamp: Option<u64>,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            max_cycles: 10_000_000,
            proof_mode: ProofMode::Sp1,
            allow_mock_fallback: false,
            deterministic_mock_timestamp: None,
        }
    }
}

/// Neo zkVM Prover
pub struct NeoProver {
    config: ProverConfig,
}

impl NeoProver {
    /// Check if the SP1 ELF is available and valid
    pub fn is_elf_available() -> bool {
        !NEO_ZKVM_ELF.is_empty() && NEO_ZKVM_ELF.len() > 100 && !NEO_ZKVM_ELF.starts_with(b"DUMMY")
    }

    fn sp1_unavailable_reason() -> &'static str {
        Self::sp1_unavailable_reason_for_elf(NEO_ZKVM_ELF)
    }

    fn sp1_unavailable_reason_for_elf(elf: &[u8]) -> &'static str {
        if elf.starts_with(elf_markers::DUMMY_ELF_NO_PROGRAM_SOURCE) {
            "SP1 prover was built without neo-zkvm-program source"
        } else if elf.starts_with(elf_markers::DUMMY_ELF_NOT_FOR_PRODUCTION) {
            "SP1 toolchain is not installed"
        } else if elf.starts_with(elf_markers::DUMMY_ELF_FOR_CLIPPY) {
            "SP1 ELF build was skipped for clippy/analysis"
        } else if elf.starts_with(elf_markers::DUMMY_ELF_BUILD_FAILED) {
            "SP1 guest ELF build failed"
        } else {
            "SP1 ELF is unavailable"
        }
    }

    /// Create a new prover with the given configuration
    pub fn new(config: ProverConfig) -> Self {
        Self { config }
    }

    /// Maximum allowed script size — imported from `neo_vm_core`.
    const MAX_SCRIPT_SIZE: usize = neo_vm_core::MAX_SCRIPT_SIZE;
    /// Keep failed-proof error payloads small and deterministic.
    const MAX_FAILED_PROOF_ERROR_BYTES: usize = 8 * 1024;

    fn sanitize_failed_proof_error(mut error: String) -> String {
        if error.len() <= Self::MAX_FAILED_PROOF_ERROR_BYTES {
            return error;
        }

        let mut boundary = Self::MAX_FAILED_PROOF_ERROR_BYTES;
        while boundary > 0 && !error.is_char_boundary(boundary) {
            boundary -= 1;
        }
        // If boundary hit 0 (all multi-byte chars at limit), keep at least one char
        if boundary == 0 {
            boundary = error.char_indices().nth(1).map_or(error.len(), |(i, _)| i);
        }
        let omitted_bytes = error.len().saturating_sub(boundary);
        error.truncate(boundary);
        error.push_str(&format!("... [truncated {omitted_bytes} bytes]"));
        error
    }

    fn safe_output_hash(output: &ProofOutput) -> [u8; 32] {
        match try_hash_proof_output(output) {
            Ok(hash) => hash,
            Err(err) => {
                let mut fallback = b"neo-zkvm:failed-proof-output:v1:".to_vec();
                fallback.extend_from_slice(err.to_string().as_bytes());
                hash_data(&fallback)
            }
        }
    }

    fn failed_proof(&self, script_hash: [u8; 32], input_hash: [u8; 32], error: String) -> NeoProof {
        let sanitized_error = Self::sanitize_failed_proof_error(error);
        let output = ProofOutput {
            state: 1,
            result: Some(neo_vm_core::StackItem::Boolean(false)),
            gas_consumed: 0,
            error: Some(sanitized_error),
        };
        let public_inputs = PublicInputs {
            script_hash,
            input_hash,
            output_hash: Self::safe_output_hash(&output),
            gas_consumed: output.gas_consumed,
            execution_success: false,
        };

        NeoProof {
            output,
            proof_bytes: vec![],
            public_inputs,
            vkey_hash: [0u8; 32],
            proof_mode: self.config.proof_mode,
            proof_format_version: PROOF_FORMAT_VERSION,
        }
    }

    fn fallback_artifacts(&self, public_inputs: &PublicInputs, warning: &str) -> Sp1ProofArtifacts {
        eprintln!("Warning: {warning}");
        (
            self.generate_mock_proof(public_inputs),
            [0u8; 32],
            public_inputs.clone(),
        )
    }

    /// Try generating an SP1 proof in the given mode, falling back to mock or
    /// returning a failed proof depending on `allow_mock_fallback`.
    fn try_sp1_proof_or_fallback(
        &self,
        input: &ProofInput,
        sp1_mode: SP1ProofMode,
        neo_mode: ProofMode,
        script_hash: [u8; 32],
        input_hash: [u8; 32],
        public_inputs: &PublicInputs,
    ) -> Result<Sp1FallbackResult, Box<NeoProof>> {
        match self.generate_sp1_proof(input, sp1_mode) {
            Ok((bytes, hash, inputs)) => Ok((bytes, hash, neo_mode, Some(inputs))),
            Err(err) => {
                let warning = format!(
                    "{neo_mode:?} proof generation failed ({err}), {}",
                    if self.config.allow_mock_fallback {
                        "falling back to mock"
                    } else {
                        "and mock fallback is disabled"
                    }
                );
                if self.config.allow_mock_fallback {
                    let (bytes, hash, inputs) = self.fallback_artifacts(public_inputs, &warning);
                    Ok((bytes, hash, ProofMode::Mock, Some(inputs)))
                } else {
                    Err(Box::new(self.failed_proof(
                        script_hash,
                        input_hash,
                        warning,
                    )))
                }
            }
        }
    }

    /// Hash a serialized `ProofInput` for public input commitment.
    pub fn hash_proof_input(input: &ProofInput) -> Result<[u8; 32], bincode::Error> {
        let bytes = bincode_options().serialize(input)?;
        Ok(hash_data(&bytes))
    }

    /// Generate a proof for the given input.
    pub fn prove(&self, mut input: ProofInput) -> NeoProof {
        if input.script.len() > Self::MAX_SCRIPT_SIZE {
            return self.failed_proof(
                hash_data(&input.script),
                [0u8; 32],
                format!(
                    "Script size {} bytes exceeds maximum allowed size of {} bytes",
                    input.script.len(),
                    Self::MAX_SCRIPT_SIZE,
                ),
            );
        }

        if input.gas_limit > self.config.max_cycles {
            eprintln!(
                "Warning: gas limit {} exceeds configured max_cycles {}; capping.",
                input.gas_limit, self.config.max_cycles
            );
            input.gas_limit = self.config.max_cycles;
        }

        let script_hash = hash_data(&input.script);
        let input_hash = match Self::hash_proof_input(&input) {
            Ok(hash) => hash,
            Err(err) => {
                return self.failed_proof(
                    script_hash,
                    [0u8; 32],
                    format!("Failed to serialize proof input: {err}"),
                );
            }
        };

        let output = execute(input.clone());
        let output_hash = match try_hash_proof_output(&output) {
            Ok(hash) => hash,
            Err(err) => {
                return self.failed_proof(
                    script_hash,
                    input_hash,
                    format!("Failed to serialize proof output: {err}"),
                );
            }
        };

        let mut public_inputs = PublicInputs {
            script_hash,
            input_hash,
            output_hash,
            gas_consumed: output.gas_consumed,
            execution_success: output.state == 0,
        };

        let sp1_available = Self::is_elf_available();

        let (proof_bytes, vkey_hash, actual_mode, sp1_public_inputs) = match self.config.proof_mode
        {
            ProofMode::Execute => (vec![], [0u8; 32], ProofMode::Execute, None),
            ProofMode::Mock => (
                self.generate_mock_proof(&public_inputs),
                [0u8; 32],
                ProofMode::Mock,
                None,
            ),
            ProofMode::Sp1 if sp1_available => {
                match self.try_sp1_proof_or_fallback(
                    &input,
                    SP1ProofMode::Compressed,
                    ProofMode::Sp1,
                    script_hash,
                    input_hash,
                    &public_inputs,
                ) {
                    Ok(result) => result,
                    Err(failed) => return *failed,
                }
            }
            ProofMode::Plonk if sp1_available => {
                match self.try_sp1_proof_or_fallback(
                    &input,
                    SP1ProofMode::Plonk,
                    ProofMode::Plonk,
                    script_hash,
                    input_hash,
                    &public_inputs,
                ) {
                    Ok(result) => result,
                    Err(failed) => return *failed,
                }
            }
            ProofMode::Groth16 if sp1_available => {
                match self.try_sp1_proof_or_fallback(
                    &input,
                    SP1ProofMode::Groth16,
                    ProofMode::Groth16,
                    script_hash,
                    input_hash,
                    &public_inputs,
                ) {
                    Ok(result) => result,
                    Err(failed) => return *failed,
                }
            }
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16 => {
                let warning = format!(
                    "{}; {}",
                    Self::sp1_unavailable_reason(),
                    if self.config.allow_mock_fallback {
                        "falling back to mock proof"
                    } else {
                        "mock fallback is disabled"
                    }
                );
                if self.config.allow_mock_fallback {
                    let (bytes, hash, inputs) = self.fallback_artifacts(&public_inputs, &warning);
                    (bytes, hash, ProofMode::Mock, Some(inputs))
                } else {
                    return self.failed_proof(script_hash, input_hash, warning);
                }
            }
        };

        if let Some(inputs) = sp1_public_inputs {
            if !public_inputs_equal(&inputs, &public_inputs) {
                let warning = format!(
                    "SP1 public inputs differ from host execution; {}",
                    if self.config.allow_mock_fallback {
                        "falling back to mock"
                    } else {
                        "mock fallback is disabled"
                    }
                );
                if self.config.allow_mock_fallback {
                    let (bytes, hash, _) = self.fallback_artifacts(&public_inputs, &warning);
                    return NeoProof {
                        output,
                        proof_bytes: bytes,
                        public_inputs,
                        vkey_hash: hash,
                        proof_mode: ProofMode::Mock,
                        proof_format_version: PROOF_FORMAT_VERSION,
                    };
                }
                return self.failed_proof(script_hash, input_hash, warning);
            }
            public_inputs = inputs;
        }

        NeoProof {
            output,
            proof_bytes,
            public_inputs,
            vkey_hash,
            proof_mode: actual_mode,
            proof_format_version: PROOF_FORMAT_VERSION,
        }
    }

    /// Generate a proof and fail if requested cryptographic mode falls back.
    pub fn prove_strict(&self, input: ProofInput) -> Result<NeoProof, String> {
        let requested_mode = self.config.proof_mode;
        let proof = self.prove(input);
        if matches!(
            requested_mode,
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16
        ) {
            if proof.proof_mode != requested_mode {
                return Err(format!(
                    "Requested proof mode {:?} but prover produced {:?}. Re-run with fallback enabled if this is expected.",
                    requested_mode, proof.proof_mode
                ));
            }
            if proof.output.state != 0 {
                return Err(proof.output.error.clone().unwrap_or_else(|| {
                    format!("Proof generation in {:?} mode failed", requested_mode)
                }));
            }
        }
        Ok(proof)
    }

    /// Verify a proof
    pub fn verify(&self, proof: &NeoProof) -> bool {
        if proof.proof_format_version != PROOF_FORMAT_VERSION {
            return false;
        }
        if !output_matches_public_inputs(proof) {
            return false;
        }
        match proof.proof_mode {
            ProofMode::Execute => proof.output.state == 0,
            ProofMode::Mock => proof.output.state == 0 && self.verify_mock_proof(proof),
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16 => {
                proof.output.state == 0 && self.verify_sp1_proof(proof).unwrap_or(false)
            }
        }
    }

    fn generate_mock_proof(&self, inputs: &PublicInputs) -> Vec<u8> {
        let timestamp = self.config.deterministic_mock_timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        self.create_mock_proof(inputs, timestamp)
    }

    fn create_mock_proof(&self, inputs: &PublicInputs, timestamp: u64) -> Vec<u8> {
        let mock = MockProof {
            public_inputs: inputs.clone(),
            commitment: compute_commitment(inputs),
            timestamp,
        };
        bincode_options()
            .serialize(&mock)
            .expect("MockProof serialization must not fail")
    }

    fn verify_mock_proof(&self, proof: &NeoProof) -> bool {
        match bincode_options().deserialize::<MockProof>(&proof.proof_bytes) {
            Ok(mock) => {
                let expected = compute_commitment(&proof.public_inputs);
                mock.commitment == expected
                    && public_inputs_equal(&mock.public_inputs, &proof.public_inputs)
            }
            Err(_) => false,
        }
    }

    fn generate_sp1_proof(
        &self,
        input: &ProofInput,
        mode: SP1ProofMode,
    ) -> Result<Sp1ProofArtifacts, DynError> {
        if cfg!(debug_assertions) {
            return Err("SP1 proving requires a release build (--release)".into());
        }
        if !Self::is_elf_available() {
            return Err(Self::sp1_unavailable_reason().into());
        }

        let prover = ProverClient::from_env();
        let (pk, vk) = prover.setup(NEO_ZKVM_ELF);
        let stdin = self.prepare_stdin(input);

        let proof = match mode {
            SP1ProofMode::Core => prover.prove(&pk, &stdin).core().run(),
            SP1ProofMode::Compressed => prover.prove(&pk, &stdin).compressed().run(),
            SP1ProofMode::Plonk => prover.prove(&pk, &stdin).plonk().run(),
            SP1ProofMode::Groth16 => prover.prove(&pk, &stdin).groth16().run(),
        }?;

        prover.verify(&proof, &vk)?;

        let public_inputs = decode_public_inputs(&proof.public_values)?;
        let proof_bytes = bincode_options().serialize(&proof)?;
        let vkey_hash = hash_data(&bincode_options().serialize(&vk)?);

        Ok((proof_bytes, vkey_hash, public_inputs))
    }

    fn verify_sp1_proof(&self, proof: &NeoProof) -> Result<bool, DynError> {
        if !Self::is_elf_available() {
            return Ok(false);
        }

        let prover = ProverClient::from_env();
        let (_, vk) = prover.setup(NEO_ZKVM_ELF);
        let expected_vkey_hash = hash_data(&bincode_options().serialize(&vk)?);
        if expected_vkey_hash != proof.vkey_hash {
            return Ok(false);
        }

        let sp1_proof: sp1_sdk::SP1ProofWithPublicValues =
            bincode_options().deserialize(&proof.proof_bytes)?;
        let pi = decode_public_inputs(&sp1_proof.public_values)?;
        if !public_inputs_equal(&pi, &proof.public_inputs) {
            return Ok(false);
        }

        Ok(prover.verify(&sp1_proof, &vk).is_ok())
    }

    fn prepare_stdin(&self, input: &ProofInput) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();
        stdin.write(input);
        stdin
    }
}

fn decode_public_inputs(values: &SP1PublicValues) -> Result<PublicInputs, DynError> {
    Ok(bincode_options().deserialize(values.as_slice())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_vm_core::StackItem;

    #[test]
    fn test_mock_proof() {
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
        assert!(proof.proof_mode == ProofMode::Mock);
        assert!(prover.verify(&proof));
    }

    #[test]
    fn test_execute_only() {
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
        assert!(proof.proof_mode == ProofMode::Execute);
        assert!(prover.verify(&proof));
    }

    #[test]
    fn test_prove_strict_allows_execute_mode() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Execute,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover
            .prove_strict(input)
            .expect("execute mode should be strict-safe");
        assert_eq!(proof.proof_mode, ProofMode::Execute);
    }

    #[test]
    fn test_prove_strict_rejects_crypto_fallback() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Sp1,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let result = prover.prove_strict(input);
        if cfg!(debug_assertions) || !NeoProver::is_elf_available() {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_prove_sp1_mode_fails_closed_in_debug_without_fallback_opt_in() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Sp1,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        if cfg!(debug_assertions) {
            assert_ne!(proof.output.state, 0);
            assert_eq!(proof.proof_mode, ProofMode::Sp1);
        }
    }

    #[test]
    fn test_prove_sp1_mode_supports_explicit_fallback_opt_in() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Sp1,
            allow_mock_fallback: true,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        if cfg!(debug_assertions) || !NeoProver::is_elf_available() {
            assert_eq!(proof.proof_mode, ProofMode::Mock);
            assert_eq!(proof.output.state, 0);
        }
    }

    #[test]
    fn test_input_hash_matches_serialized_proof_input() {
        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![StackItem::Integer(7)],
            gas_limit: 123,
        };

        let bytes = bincode_options().serialize(&input).expect("serialize");
        let hash = hash_data(&bytes);

        assert_eq!(
            hash,
            NeoProver::hash_proof_input(&input).expect("hash_proof_input should succeed")
        );
    }

    #[test]
    fn test_input_hash_distinguishes_complex_argument_variants() {
        let null_input = ProofInput {
            script: vec![0x40],
            arguments: vec![StackItem::Null],
            gas_limit: 1_000_000,
        };
        let array_input = ProofInput {
            script: vec![0x40],
            arguments: vec![StackItem::Array(vec![StackItem::Integer(42)])],
            gas_limit: 1_000_000,
        };

        assert_ne!(
            NeoProver::hash_proof_input(&null_input).expect("null input should hash"),
            NeoProver::hash_proof_input(&array_input).expect("array input should hash")
        );
    }

    #[test]
    fn test_hash_proof_input_rejects_oversized_arguments() {
        let input = ProofInput {
            script: vec![0x40],
            arguments: vec![StackItem::ByteString(vec![0u8; 10 * 1024 * 1024 + 1])],
            gas_limit: 1_000_000,
        };

        assert!(NeoProver::hash_proof_input(&input).is_err());
    }

    #[test]
    fn test_prove_rejects_unhashable_input_arguments() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let input = ProofInput {
            script: vec![0x40],
            arguments: vec![StackItem::ByteString(vec![0u8; 10 * 1024 * 1024 + 1])],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        assert_ne!(proof.output.state, 0);
        assert!(proof
            .output
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Failed to serialize proof input"));
    }

    #[test]
    fn test_mock_proof_tamper_detection() {
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
        proof.public_inputs.script_hash[0] ^= 0xFF;
        assert!(!prover.verify(&proof));
    }

    #[test]
    fn test_proof_output_contains_result() {
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
        assert_eq!(proof.output.state, 0);
        assert!(proof.output.gas_consumed > 0);
    }

    #[test]
    fn test_output_tampering_is_detected() {
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
        assert!(!prover.verify(&proof));
    }

    #[test]
    fn test_faulting_script_produces_failed_proof() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let input = ProofInput {
            script: vec![0x15, 0x10, 0xA1, 0x40], // div by zero
            arguments: vec![],
            gas_limit: 1_000_000,
        };
        let proof = prover.prove(input);
        assert_ne!(proof.output.state, 0);
    }

    #[test]
    fn test_verify_execute_faulted_proof_rejected() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Execute,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x15, 0x10, 0xA1, 0x40], // div by zero
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        assert_ne!(proof.output.state, 0);
        assert!(!prover.verify(&proof));
    }

    #[test]
    fn test_verify_mock_faulted_proof_rejected() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x15, 0x10, 0xA1, 0x40], // div by zero
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = prover.prove(input);
        assert_ne!(proof.output.state, 0);
        assert!(!prover.verify(&proof));
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

        assert!(!prover.verify(&proof));
    }

    #[test]
    fn test_max_cycles_caps_effective_gas_limit() {
        let capped = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Execute,
            max_cycles: 5,
            ..Default::default()
        });
        let uncapped = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Execute,
            max_cycles: 1_000_000,
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x40], // many PUSH1 + RET
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof = capped.prove(input.clone());
        let proof_uncapped = uncapped.prove(input);
        assert_ne!(
            proof.output.state, 0,
            "execution should fault under capped gas"
        );
        assert_eq!(proof_uncapped.output.state, 0);
    }

    #[test]
    fn test_deterministic_mock_proof_timestamp_produces_stable_bytes() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            deterministic_mock_timestamp: Some(42),
            ..Default::default()
        });

        let input = ProofInput {
            script: vec![0x12, 0x13, 0x9E, 0x40],
            arguments: vec![],
            gas_limit: 1_000_000,
        };

        let proof_a = prover.prove(input.clone());
        let proof_b = prover.prove(input);
        assert_eq!(proof_a.proof_bytes, proof_b.proof_bytes);
    }

    #[test]
    fn test_failed_proof_does_not_panic_on_oversized_error_message() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });
        let oversized_error = "x".repeat(10 * 1024 * 1024 + 1024);

        let result =
            std::panic::catch_unwind(|| prover.failed_proof([1u8; 32], [2u8; 32], oversized_error));

        assert!(
            result.is_ok(),
            "failed_proof should never panic on oversized error messages"
        );
        let proof = result.expect("proof should be returned");
        assert_eq!(proof.output.state, 1);
        assert_eq!(proof.public_inputs.gas_consumed, 0);
        assert!(!proof.public_inputs.execution_success);
        assert!(proof
            .output
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("truncated"));
    }

    #[test]
    fn test_sp1_unavailable_reason_reports_build_failed_marker() {
        assert_eq!(
            NeoProver::sp1_unavailable_reason_for_elf(b"DUMMY_ELF_BUILD_FAILED"),
            "SP1 guest ELF build failed"
        );
    }

    #[test]
    fn test_sp1_unavailable_reason_reports_toolchain_marker() {
        assert_eq!(
            NeoProver::sp1_unavailable_reason_for_elf(b"DUMMY_ELF_NOT_FOR_PRODUCTION"),
            "SP1 toolchain is not installed"
        );
    }

    #[test]
    fn test_hash_proof_input_does_not_panic_on_oversized_arguments() {
        let input = ProofInput {
            script: vec![0x40],
            arguments: vec![StackItem::ByteString(vec![0u8; 10 * 1024 * 1024 + 1])],
            gas_limit: 1_000_000,
        };

        let result = std::panic::catch_unwind(|| NeoProver::hash_proof_input(&input));
        assert!(result.is_ok(), "hash_proof_input should be panic-free");
        assert!(result.expect("result should be returned").is_err());
    }
}
