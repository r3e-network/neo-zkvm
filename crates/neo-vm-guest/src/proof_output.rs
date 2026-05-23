use serde::{Deserialize, Serialize};

use super::StackItem;

/// Output from zkVM execution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofOutput {
    /// VM exit state: 0 = halt (success), non-zero = fault.
    pub state: u8,
    /// Top-of-stack value at halt, if any.
    pub result: Option<StackItem>,
    /// Total gas consumed during execution.
    pub gas_consumed: u64,
    /// Human-readable error message on fault.
    pub error: Option<String>,
}
