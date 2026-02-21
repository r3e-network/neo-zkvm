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
use neo_vm_guest::{
    bincode_options, compute_commitment, execute, hash_data, hash_proof_output,
    output_matches_public_inputs, public_inputs_equal, try_hash_proof_output, ProofInput,
    ProofOutput,
};
use sp1_sdk::{ProverClient, SP1ProofMode, SP1PublicValues, SP1Stdin};

// Re-export shared types so downstream crates (CLI, examples) keep compiling.
pub use neo_vm_guest::{MockProof, NeoProof, ProofMode, PublicInputs, PROOF_FORMAT_VERSION};

/// SP1 ELF binary - embedded at compile time.
pub const NEO_ZKVM_ELF: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/elf/riscv32im-succinct-zkvm-elf"));

type DynError = Box<dyn std::error::Error>;
type Sp1ProofArtifacts = (Vec<u8>, [u8; 32], PublicInputs);

/// Prover configuration
#[derive(Clone, Debug)]
pub struct ProverConfig {
    /// Maximum cycles for SP1 execution
    pub max_cycles: u64,
    /// Proof mode (determines proof type and verification cost)
    pub proof_mode: ProofMode,
    /// Optional fixed timestamp for deterministic mock proofs.
    pub deterministic_mock_timestamp: Option<u64>,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            max_cycles: 10_000_000,
            proof_mode: ProofMode::Sp1,
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
        if NEO_ZKVM_ELF.starts_with(b"DUMMY_ELF_NO_PROGRAM_SOURCE") {
            "SP1 prover was built without neo-zkvm-program source"
        } else if NEO_ZKVM_ELF.starts_with(b"DUMMY_ELF_NOT_FOR_PRODUCTION") {
            "SP1 toolchain is not installed"
        } else if NEO_ZKVM_ELF.starts_with(b"DUMMY_ELF_FOR_CLIPPY") {
            "SP1 ELF build was skipped for clippy/analysis"
        } else {
            "SP1 ELF is unavailable"
        }
    }

    /// Create a new prover with the given configuration
    pub fn new(config: ProverConfig) -> Self {
        Self { config }
    }

    /// Maximum allowed script size (1 MB).
    const MAX_SCRIPT_SIZE: usize = 1024 * 1024;

    fn failed_proof(&self, script_hash: [u8; 32], input_hash: [u8; 32], error: String) -> NeoProof {
        let output = ProofOutput {
            state: 1,
            result: Some(neo_vm_core::StackItem::Boolean(false)),
            gas_consumed: 0,
            error: Some(error),
        };
        let public_inputs = PublicInputs {
            script_hash,
            input_hash,
            output_hash: hash_proof_output(&output),
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

    /// Hash a serialized `ProofInput` for public input commitment.
    pub fn try_hash_proof_input(input: &ProofInput) -> Result<[u8; 32], bincode::Error> {
        let bytes = bincode_options().serialize(input)?;
        Ok(hash_data(&bytes))
    }

    /// Hash a serialized `ProofInput` for public input commitment.
    ///
    /// This panics if serialization fails; prefer `try_hash_proof_input` in fallible paths.
    pub fn hash_proof_input(input: &ProofInput) -> [u8; 32] {
        Self::try_hash_proof_input(input)
            .expect("failed to serialize ProofInput for hashing; use try_hash_proof_input")
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
        let input_hash = match Self::try_hash_proof_input(&input) {
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
                match self.generate_sp1_proof(&input, SP1ProofMode::Compressed) {
                    Ok((bytes, hash, inputs)) => (bytes, hash, ProofMode::Sp1, Some(inputs)),
                    Err(err) => {
                        eprintln!(
                            "Warning: SP1 proof generation failed ({err}), falling back to mock"
                        );
                        (
                            self.generate_mock_proof(&public_inputs),
                            [0u8; 32],
                            ProofMode::Mock,
                            None,
                        )
                    }
                }
            }
            ProofMode::Plonk if sp1_available => {
                match self.generate_sp1_proof(&input, SP1ProofMode::Plonk) {
                    Ok((bytes, hash, inputs)) => (bytes, hash, ProofMode::Plonk, Some(inputs)),
                    Err(err) => {
                        eprintln!(
                            "Warning: PLONK proof generation failed ({err}), falling back to mock"
                        );
                        (
                            self.generate_mock_proof(&public_inputs),
                            [0u8; 32],
                            ProofMode::Mock,
                            None,
                        )
                    }
                }
            }
            ProofMode::Groth16 if sp1_available => {
                match self.generate_sp1_proof(&input, SP1ProofMode::Groth16) {
                    Ok((bytes, hash, inputs)) => (bytes, hash, ProofMode::Groth16, Some(inputs)),
                    Err(err) => {
                        eprintln!("Warning: Groth16 proof generation failed ({err}), falling back to mock");
                        (
                            self.generate_mock_proof(&public_inputs),
                            [0u8; 32],
                            ProofMode::Mock,
                            None,
                        )
                    }
                }
            }
            _ => {
                eprintln!(
                    "Warning: {}. Falling back to mock proof.",
                    Self::sp1_unavailable_reason()
                );
                (
                    self.generate_mock_proof(&public_inputs),
                    [0u8; 32],
                    ProofMode::Mock,
                    None,
                )
            }
        };

        if let Some(inputs) = sp1_public_inputs {
            if !public_inputs_equal(&inputs, &public_inputs) {
                eprintln!(
                    "Warning: SP1 public inputs differ from host execution; falling back to mock"
                );
                return NeoProof {
                    output,
                    proof_bytes: self.generate_mock_proof(&public_inputs),
                    public_inputs,
                    vkey_hash: [0u8; 32],
                    proof_mode: ProofMode::Mock,
                    proof_format_version: PROOF_FORMAT_VERSION,
                };
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
        ) && proof.proof_mode != requested_mode
        {
            return Err(format!(
                "Requested proof mode {:?} but prover produced {:?}. Re-run with fallback enabled if this is expected.",
                requested_mode, proof.proof_mode
            ));
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
        bincode_options().serialize(&mock).unwrap_or_default()
    }

    fn verify_mock_proof(&self, proof: &NeoProof) -> bool {
        match bincode_options().deserialize::<MockProof>(&proof.proof_bytes) {
            Ok(mock) => {
                let expected = compute_commitment(&proof.public_inputs);
                mock.commitment == expected
                    && mock.public_inputs.script_hash == proof.public_inputs.script_hash
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
        if cfg!(debug_assertions) {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
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

        assert_eq!(hash, NeoProver::hash_proof_input(&input));
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
            NeoProver::hash_proof_input(&null_input),
            NeoProver::hash_proof_input(&array_input)
        );
    }

    #[test]
    fn test_try_hash_proof_input_rejects_oversized_arguments() {
        let input = ProofInput {
            script: vec![0x40],
            arguments: vec![StackItem::ByteString(vec![0u8; 10 * 1024 * 1024 + 1])],
            gas_limit: 1_000_000,
        };

        assert!(NeoProver::try_hash_proof_input(&input).is_err());
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
}
