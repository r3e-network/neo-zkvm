use neo_vm_guest::ProofMode;

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
            // Align with CLI: Sp1 only when the crate is built with the sp1 feature.
            proof_mode: if cfg!(feature = "sp1") {
                ProofMode::Sp1
            } else {
                ProofMode::Mock
            },
            allow_mock_fallback: false,
            deterministic_mock_timestamp: None,
        }
    }
}
