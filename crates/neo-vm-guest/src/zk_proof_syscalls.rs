use neo_vm_rs::SyscallProvider;
use sha2::{Digest, Sha256};

use super::{interop_hash, pop_byte_arg, StackItem};

pub(super) struct ZkProofSyscalls;

impl SyscallProvider for ZkProofSyscalls {
    fn syscall(&mut self, api: u32, _ip: usize, stack: &mut Vec<StackItem>) -> Result<(), String> {
        if api == interop_hash("System.Crypto.SHA256") {
            let bytes = pop_byte_arg(stack, "System.Crypto.SHA256")?;
            stack.push(StackItem::ByteString(Sha256::digest(&bytes).to_vec()));
            return Ok(());
        }

        Err(format!(
            "unsupported zk proof syscall 0x{api:08x}; provide a deterministic zk syscall adapter"
        ))
    }
}
