use serde::{Deserialize, Serialize};

use super::PublicInputs;

/// Mock proof structure for testing.
#[derive(Serialize, Deserialize)]
pub struct MockProof {
    /// Public inputs used to compute the commitment.
    pub public_inputs: PublicInputs,
    /// SHA-256 commitment over the public inputs (double-SHA256 of the
    /// canonical encoding, matching Neo's `Crypto.Hash256`).
    pub commitment: [u8; 32],
    /// Unix timestamp when the mock proof was created.
    pub timestamp: u64,
}
