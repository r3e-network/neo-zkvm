//! Fuzz target for VM execution
//!
//! Tests VM execution with arbitrary bytecode.

#![no_main]

use libfuzzer_sys::fuzz_target;
use neo_vm_core::{NeoVM, VMState};

fuzz_target!(|data: &[u8]| {
    // Skip empty input
    if data.is_empty() {
        return;
    }

    // Create VM with limited gas to prevent infinite loops
    let mut vm = NeoVM::new(10_000);
    
    // Append RET opcode to ensure termination
    let mut script = data.to_vec();
    script.push(0x40); // RET
    
    vm.load_script(script);

    // Execute until halt or fault
    let mut steps = 0;
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if vm.execute_next().is_err() {
            break;
        }
        steps += 1;
        if steps > 1000 {
            break;
        }
    }
});
