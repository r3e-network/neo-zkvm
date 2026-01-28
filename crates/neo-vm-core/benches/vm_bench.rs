//! Benchmarks for Neo VM Core

use neo_vm_core::{NeoVM, VMState};
use std::time::Instant;

fn benchmark_arithmetic() {
    let script = vec![
        0x12, 0x13, 0x9E, // PUSH2, PUSH3, ADD
        0x14, 0xA0,       // PUSH4, MUL
        0x12, 0xA1,       // PUSH2, DIV
        0x40,             // RET
    ];
    
    let iterations = 10000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let mut vm = NeoVM::new(1_000_000);
        vm.load_script(script.clone());
        while !matches!(vm.state, VMState::Halt | VMState::Fault) {
            let _ = vm.execute_next();
        }
    }
    
    let elapsed = start.elapsed();
    println!("Arithmetic: {} iterations in {:?}", iterations, elapsed);
    println!("  Per iteration: {:?}", elapsed / iterations);
}

fn main() {
    println!("Neo VM Benchmarks\n");
    benchmark_arithmetic();
}
