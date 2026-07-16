//! Fuzz structured scripts with Arbitrary-derived gas + integer stack args.

#![no_main]

mod common;

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
    let gas_limit = common::clamp_gas(input.gas_limit as u64);

    let arguments: Vec<StackItem> = input
        .initial_stack
        .iter()
        .take(16)
        .map(|value| StackItem::Integer(*value))
        .collect();

    let script = common::build_structured_script(&input.script, 256);

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit,
    });
});
