use serde::Deserialize;

use super::{ProofMode, ProofOutput, PublicInputs};

#[derive(Deserialize)]
pub struct LegacyNeoProof {
    pub output: ProofOutput,
    pub proof_bytes: Vec<u8>,
    pub public_inputs: PublicInputs,
    pub vkey_hash: [u8; 32],
    pub proof_mode: ProofMode,
}
