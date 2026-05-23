//! Fuzz target for script parsing
//!
//! Tests script parsing with arbitrary input.

#![no_main]

mod common;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, OpCode, ProofInput, StackItem};

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

    let mut script = Vec::with_capacity(input.script.len().min(256) * 5 + 1);
    for byte in input.script.into_iter().take(256) {
        common::append_bounded_neo_vm_sequence(&mut script, byte);
    }
    script.push(OpCode::RET.byte());

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit,
    });
});
