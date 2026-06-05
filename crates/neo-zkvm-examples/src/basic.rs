//! Basic shared VM usage example
use neo_vm_guest::{OpCode, ProofInput, execute};

fn main() {
    // 2 + 3 = 5
    let output = execute(ProofInput {
        script: vec![
            OpCode::PUSH2.byte(),
            OpCode::PUSH3.byte(),
            OpCode::ADD.byte(),
            OpCode::RET.byte(),
        ],
        arguments: vec![],
        gas_limit: 1_000_000,
    });

    println!("Result: {:?}", output.result);
}
