//! Neo VM Guest Program for zkVM proving

use neo_vm_core::{NeoVM, StackItem, VMState};
use serde::{Deserialize, Serialize};

/// Input for zkVM proving
#[derive(Serialize, Deserialize, Clone)]
pub struct ProofInput {
    pub script: Vec<u8>,
    pub arguments: Vec<StackItem>,
    pub gas_limit: u64,
}

/// Output from zkVM execution
#[derive(Serialize, Deserialize)]
pub struct ProofOutput {
    pub state: u8,
    pub result: Option<StackItem>,
    pub gas_consumed: u64,
}

/// Execute Neo VM and return proof output
pub fn execute(input: ProofInput) -> ProofOutput {
    let mut vm = NeoVM::new(input.gas_limit);
    vm.load_script(input.script);

    // Push arguments
    for arg in input.arguments {
        vm.eval_stack.push(arg);
    }

    // Execute until halt or fault
    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
        if vm.execute_next().is_err() {
            vm.state = VMState::Fault;
            break;
        }
    }

    let state = match vm.state {
        VMState::Halt => 0,
        VMState::Fault => 1,
        _ => 2,
    };

    ProofOutput {
        state,
        result: vm.eval_stack.pop(),
        gas_consumed: vm.gas_consumed,
    }
}
