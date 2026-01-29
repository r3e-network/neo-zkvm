//! Fuzz target for script parsing
//!
//! Tests script parsing with arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use neo_vm_core::{NeoVM, VMState, StackItem};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    script: Vec<u8>,
    gas_limit: u32,
    initial_stack: Vec<i64>,
}

fuzz_target!(|input: FuzzInput| {
    // Limit gas to prevent long runs
    let gas = (input.gas_limit % 10_000) as u64 + 100;
    
    let mut vm = NeoVM::new(gas);
    
    // Add initial stack items
    for val in input.initial_stack.iter().take(10) {
        vm.eval_stack.push(StackItem::Integer(*val as i128));
    }
    
    // Append RET to script
    let mut script = input.script;
    if script.len() > 1000 {
        script.truncate(1000);
    }
    script.push(0x40);
    
    vm.load_script(script);

    let mut steps = 0;
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if vm.execute_next().is_err() {
            break;
        }
        steps += 1;
        if steps > 500 {
            break;
        }
    }
});
