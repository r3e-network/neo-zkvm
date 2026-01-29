//! Comprehensive Neo VM Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use neo_vm_core::{NeoVM, VMState};

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");
    
    // ADD benchmark
    group.bench_function("add", |b| {
        b.iter(|| {
            let mut vm = NeoVM::new(1_000_000);
            vm.load_script(vec![0x12, 0x13, 0x9E, 0x40]);
            while !matches!(vm.state, VMState::Halt | VMState::Fault) {
                vm.execute_next().unwrap();
            }
            black_box(vm.eval_stack.pop())
        })
    });

    // MUL benchmark
    group.bench_function("mul", |b| {
        b.iter(|| {
            let mut vm = NeoVM::new(1_000_000);
            vm.load_script(vec![0x16, 0x17, 0xA0, 0x40]);
            while !matches!(vm.state, VMState::Halt | VMState::Fault) {
                vm.execute_next().unwrap();
            }
            black_box(vm.eval_stack.pop())
        })
    });

    // DIV benchmark
    group.bench_function("div", |b| {
        b.iter(|| {
            let mut vm = NeoVM::new(1_000_000);
            vm.load_script(vec![0x1F, 0x15, 0xA1, 0x40]);
            while !matches!(vm.state, VMState::Halt | VMState::Fault) {
                vm.execute_next().unwrap();
            }
            black_box(vm.eval_stack.pop())
        })
    });

    group.finish();
}

fn bench_stack_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack");
    
    group.bench_function("dup", |b| {
        b.iter(|| {
            let mut vm = NeoVM::new(1_000_000);
            vm.load_script(vec![0x15, 0x4A, 0x40]);
            while !matches!(vm.state, VMState::Halt | VMState::Fault) {
                vm.execute_next().unwrap();
            }
            black_box(vm.eval_stack.len())
        })
    });

    group.bench_function("swap", |b| {
        b.iter(|| {
            let mut vm = NeoVM::new(1_000_000);
            vm.load_script(vec![0x11, 0x12, 0x50, 0x40]);
            while !matches!(vm.state, VMState::Halt | VMState::Fault) {
                vm.execute_next().unwrap();
            }
            black_box(vm.eval_stack.pop())
        })
    });

    group.finish();
}

fn bench_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop");
    
    for iterations in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("iterations", iterations),
            iterations,
            |b, &n| {
                // Build loop script: counter, loop body, decrement, jump if not zero
                let mut script = vec![0x00, n as u8]; // PUSHINT8 n
                for _ in 0..n {
                    script.extend_from_slice(&[0x9D]); // DEC
                }
                script.push(0x40); // RET
                
                b.iter(|| {
                    let mut vm = NeoVM::new(1_000_000);
                    vm.load_script(script.clone());
                    while !matches!(vm.state, VMState::Halt | VMState::Fault) {
                        vm.execute_next().unwrap();
                    }
                    black_box(vm.gas_consumed)
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_arithmetic, bench_stack_ops, bench_loop);
criterion_main!(benches);
