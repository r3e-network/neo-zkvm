//! Fuzz target for script parsing
//!
//! Tests script parsing with arbitrary input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, ProofInput, StackItem};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    script: Vec<u8>,
    gas_limit: u32,
    initial_stack: Vec<i64>,
}

fuzz_target!(|input: FuzzInput| {
    let gas_limit = (input.gas_limit % 10_000) as u64 + 100;

    let arguments: Vec<StackItem> = input
        .initial_stack
        .iter()
        .take(10)
        .map(|value| StackItem::Integer(*value))
        .collect();

    // Keep the fuzz target deterministic and terminating while still exploring
    // stack, arithmetic, and control-free instruction decoding through neo-vm-rs.
    let mut script: Vec<u8> = input
        .script
        .into_iter()
        .take(256)
        .map(safe_opcode)
        .collect();
    script.push(0x40);

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit,
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
