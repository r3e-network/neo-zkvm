//! Basic shared VM usage example
use neo_vm_guest::{execute, ProofInput};

fn main() {
    // 2 + 3 = 5
    let output = execute(ProofInput {
        script: vec![0x12, 0x13, 0x9E, 0x40],
        arguments: vec![],
        gas_limit: 1_000_000,
    });

    println!("Result: {:?}", output.result);
}
