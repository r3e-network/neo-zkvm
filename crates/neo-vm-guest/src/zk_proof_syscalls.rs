use neo_vm_rs::SyscallProvider;

use super::{StackItem, crypto};

/// Deterministic zk-proof syscall host with runtime step metering.
///
/// Gas is charged per executed instruction via [`SyscallProvider::on_instruction`],
/// so loops and dynamic control flow cannot under-charge relative to static
/// bytecode length estimates.
///
/// Supported deterministic crypto adapters (no host state):
/// - `System.Crypto.SHA256`
/// - `System.Crypto.RIPEMD160`
/// - `System.Crypto.Hash160` — RIPEMD160(SHA256(x))
/// - `System.Crypto.Hash256` — SHA256(SHA256(x))
///
/// Host-dependent APIs (e.g. `CheckSig`, storage, time) remain unsupported so
/// proofs stay fully deterministic.
pub(super) struct ZkProofSyscalls {
    /// Instructions executed so far (also reported as `gas_consumed`).
    steps: u64,
    /// Maximum instructions allowed before out-of-gas.
    gas_limit: u64,
}

impl ZkProofSyscalls {
    pub(super) fn new(gas_limit: u64) -> Self {
        Self {
            steps: 0,
            gas_limit,
        }
    }

    #[inline]
    pub(super) fn steps(&self) -> u64 {
        self.steps
    }
}

impl SyscallProvider for ZkProofSyscalls {
    fn on_instruction(&mut self, _opcode: u8) -> Result<(), String> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.gas_limit {
            return Err("Out of gas".to_string());
        }
        Ok(())
    }

    fn syscall(&mut self, api: u32, _ip: usize, stack: &mut Vec<StackItem>) -> Result<(), String> {
        if crypto::try_crypto_syscall(api, stack)? {
            return Ok(());
        }

        Err(format!(
            "unsupported zk proof syscall 0x{api:08x}; only deterministic crypto adapters \
             (SHA256, RIPEMD160, Hash160, Hash256) are available in the proof guest"
        ))
    }
}
