use serde::{Deserialize, Serialize};

use super::StackItem;

/// Input for zkVM proving.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofInput {
    /// Raw bytecode to execute.
    pub script: Vec<u8>,
    /// Arguments passed to the script.
    pub arguments: Vec<StackItem>,
    /// Maximum gas units allowed for execution.
    pub gas_limit: u64,
}
