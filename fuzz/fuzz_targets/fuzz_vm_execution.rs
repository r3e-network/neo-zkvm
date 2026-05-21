//! Fuzz target for VM execution
//!
//! Tests VM execution with arbitrary bytecode.

#![no_main]

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, ProofInput};

fuzz_target!(|data: &[u8]| {
    // Skip empty input
    if data.is_empty() {
        return;
    }

    let mut script: Vec<u8> = data.iter().take(256).copied().map(safe_opcode).collect();
    script.push(0x40); // RET

    let _ = execute(ProofInput {
        script,
        arguments: vec![],
        gas_limit: 10_000,
    });
});

fn safe_opcode(byte: u8) -> u8 {
    match byte % 12 {
        0 => 0x10, // PUSH0
        1 => 0x11, // PUSH1
        2 => 0x12, // PUSH2
        3 => 0x13, // PUSH3
        4 => 0x4A, // DUP
        5 => 0x45, // DROP
        6 => 0x9E, // ADD
        7 => 0x9F, // SUB
        8 => 0xA0, // MUL
        9 => 0xB3, // NUMEQUAL
        10 => 0xAA, // NOT
        _ => 0x21, // NOP
    }
}
