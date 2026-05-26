use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use super::StackItem;

/// Output from zkVM execution.
///
/// The `state` field encodes the VM exit status:
/// - `0` = Halt (successful execution)
/// - `1` = Fault (execution failed)
///
/// Any other value is invalid and will cause deserialization to fail in
/// debug builds.
#[derive(Serialize, Clone, Debug)]
pub struct ProofOutput {
    /// VM exit state: 0 = halt (success), 1 = fault.
    pub state: u8,
    /// Top-of-stack value at halt, if any.
    pub result: Option<StackItem>,
    /// Total gas consumed during execution.
    pub gas_consumed: u64,
    /// Human-readable error message on fault.
    pub error: Option<String>,
}

impl<'de> Deserialize<'de> for ProofOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            state: u8,
            result: Option<StackItem>,
            gas_consumed: u64,
            error: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        // In debug builds, validate that state is only 0 or 1.
        #[cfg(debug_assertions)]
        {
            if raw.state > 1 {
                return Err(de::Error::invalid_value(
                    serde::de::Unexpected::Unsigned(raw.state as u64),
                    &"0 (Halt) or 1 (Fault)",
                ));
            }
        }

        Ok(ProofOutput {
            state: raw.state,
            result: raw.result,
            gas_consumed: raw.gas_consumed,
            error: raw.error,
        })
    }
}
