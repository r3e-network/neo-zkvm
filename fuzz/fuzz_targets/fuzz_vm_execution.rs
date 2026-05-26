//! Fuzz target for VM execution
//!
//! Tests VM execution with arbitrary bytecode.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, OpCode, ProofInput};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let mut script = Vec::with_capacity(data.len().min(256) * 5 + 1);
    for byte in data.iter().take(256) {
        common::append_bounded_neo_vm_sequence(&mut script, *byte);
    }
    script.push(OpCode::RET.byte());

    // Fuzz with non-empty arguments: extract 1-3 argument bytes from the tail
    // of the fuzz input after the script portion.
    let mut arguments = vec![];
    if data.len() > 256 {
        let arg_bytes: Vec<u8> = data[256..].to_vec();
        if !arg_bytes.is_empty() {
            arguments.push(arg_bytes);
        }
    }

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit: 10_000,
    });
});
