//! Fuzz target for VM execution
//!
//! Tests VM execution with arbitrary bytecode.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, ProofInput};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let mut script = Vec::with_capacity(data.len().min(256) * 5 + 1);
    for byte in data.iter().take(256) {
        common::append_bounded_neo_vm_sequence(&mut script, *byte);
    }
    script.push(0x40); // RET

    let _ = execute(ProofInput {
        script,
        arguments: vec![],
        gas_limit: 10_000,
    });
});
