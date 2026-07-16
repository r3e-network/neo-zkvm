use serde::{Deserialize, Serialize};

use super::PublicInputs;

/// Mock proof structure for testing.
///
/// **Not cryptographically secure.** Anyone can forge a `MockProof` for
/// arbitrary public inputs. Production verifiers must reject Mock mode.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MockProof {
    /// Public inputs used to compute the commitment.
    pub public_inputs: PublicInputs,
    /// Domain-separated commitment over the public inputs
    /// ([`crate::compute_commitment`]) — zkVM-internal, not Neo Hash256.
    pub commitment: [u8; 32],
    /// Unix timestamp when the mock proof was created.
    pub timestamp: u64,
}
