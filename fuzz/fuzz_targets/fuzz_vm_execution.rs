//! Fuzz target for VM execution
//!
//! Tests VM execution with arbitrary bytecode.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, OpCode, ProofInput, StackItem};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let mut script = Vec::with_capacity(data.len().min(256) * 5 + 1);
    for byte in data.iter().take(256) {
        common::append_bounded_neo_vm_sequence(&mut script, *byte);
    }
    script.push(OpCode::RET.byte());

    // Optional private inputs: tail bytes become a single ByteString argument.
    let mut arguments = vec![];
    if data.len() > 256 {
        let arg_bytes = data[256..].to_vec();
        if !arg_bytes.is_empty() {
            // Cap argument size to avoid pathological allocations under fuzz.
            let capped = arg_bytes.into_iter().take(1024).collect::<Vec<_>>();
            arguments.push(StackItem::ByteString(capped));
        }
    }

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit: 10_000,
    });
});
