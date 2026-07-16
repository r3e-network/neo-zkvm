use serde::{Deserialize, Serialize};

/// Public inputs committed in a proof, used for verification.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicInputs {
    /// Domain-separated double-SHA256 of the raw script (`hash_data`).
    pub script_hash: [u8; 32],
    /// Domain-separated double-SHA256 of the serialized [`crate::ProofInput`].
    pub input_hash: [u8; 32],
    /// Domain-separated double-SHA256 of the serialized [`crate::ProofOutput`].
    pub output_hash: [u8; 32],
    /// Total gas consumed during execution.
    pub gas_consumed: u64,
    /// Whether execution completed without fault.
    pub execution_success: bool,
}
