use serde::{Deserialize, Serialize};

use super::PublicInputs;

/// Mock proof structure for testing.
#[derive(Serialize, Deserialize)]
pub struct MockProof {
    /// Public inputs used to compute the commitment.
    pub public_inputs: PublicInputs,
    /// HMAC-like commitment over the public inputs.
    pub commitment: [u8; 32],
    /// Unix timestamp when the mock proof was created.
    pub timestamp: u64,
}
