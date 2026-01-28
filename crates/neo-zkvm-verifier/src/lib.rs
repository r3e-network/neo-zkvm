//! Neo zkVM Verifier

use neo_zkvm_prover::NeoProof;

/// Verify a Neo zkVM proof
pub fn verify(proof: &NeoProof) -> bool {
    // Placeholder - actual verification
    !proof.proof_bytes.is_empty() || proof.output.state == 0
}
