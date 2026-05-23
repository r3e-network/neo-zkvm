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
            proof_mode: ProofMode::Sp1,
            allow_mock_fallback: false,
            deterministic_mock_timestamp: None,
        }
    }
}
