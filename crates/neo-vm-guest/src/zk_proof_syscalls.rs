use neo_vm_rs::SyscallProvider;
// ripemd 0.1 and sha2 0.11 pull different `digest` crate majors; import Digest
// from each crate so the correct trait impl is selected.
use ripemd::{Digest as RipeDigest, Ripemd160};
use sha2::{Digest as ShaDigest, Sha256};

use super::{StackItem, interop_hash, pop_byte_arg};

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

fn sha256(bytes: &[u8]) -> [u8; 32] {
    <Sha256 as ShaDigest>::digest(bytes).into()
}

fn ripemd160(bytes: &[u8]) -> [u8; 20] {
    <Ripemd160 as RipeDigest>::digest(bytes).into()
}

fn hash160(bytes: &[u8]) -> [u8; 20] {
    ripemd160(&sha256(bytes))
}

fn hash256(bytes: &[u8]) -> [u8; 32] {
    sha256(&sha256(bytes))
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
        // Cache interop hashes once per process — they are pure functions of the name.
        let sha256_api = interop_hash("System.Crypto.SHA256");
        let ripemd_api = interop_hash("System.Crypto.RIPEMD160");
        let hash160_api = interop_hash("System.Crypto.Hash160");
        let hash256_api = interop_hash("System.Crypto.Hash256");

        if api == sha256_api {
            let bytes = pop_byte_arg(stack, "System.Crypto.SHA256")?;
            stack.push(StackItem::ByteString(sha256(&bytes).to_vec()));
            return Ok(());
        }
        if api == ripemd_api {
            let bytes = pop_byte_arg(stack, "System.Crypto.RIPEMD160")?;
            stack.push(StackItem::ByteString(ripemd160(&bytes).to_vec()));
            return Ok(());
        }
        if api == hash160_api {
            let bytes = pop_byte_arg(stack, "System.Crypto.Hash160")?;
            stack.push(StackItem::ByteString(hash160(&bytes).to_vec()));
            return Ok(());
        }
        if api == hash256_api {
            let bytes = pop_byte_arg(stack, "System.Crypto.Hash256")?;
            stack.push(StackItem::ByteString(hash256(&bytes).to_vec()));
            return Ok(());
        }

        Err(format!(
            "unsupported zk proof syscall 0x{api:08x}; only deterministic crypto adapters \
             (SHA256, RIPEMD160, Hash160, Hash256) are available in the proof guest"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
            .collect()
    }

    #[test]
    fn hash_helpers_match_known_vectors() {
        // SHA256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let empty_sha = sha256(b"");
        assert_eq!(
            empty_sha.as_slice(),
            parse_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );

        // RIPEMD160("") = 9c1185a5c5e9fc54612808977ee8f548b2258d31
        let empty_ripemd = ripemd160(b"");
        assert_eq!(
            empty_ripemd.as_slice(),
            parse_hex("9c1185a5c5e9fc54612808977ee8f548b2258d31")
        );

        // Hash160(x) = RIPEMD160(SHA256(x))
        assert_eq!(hash160(b"abc"), ripemd160(&sha256(b"abc")));
        // Hash256(x) = SHA256(SHA256(x))
        assert_eq!(hash256(b"abc"), sha256(&sha256(b"abc")));
    }
}
