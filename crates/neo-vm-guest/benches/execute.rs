//! Host-path execution benchmarks for neo-vm-guest.
//!
//! Run with:
//! ```bash
//! cargo bench -p neo-vm-guest --bench execute
//! ```

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use neo_vm_guest::{OpCode, ProofInput, StackItem, execute, interop_hash};

fn add_script() -> Vec<u8> {
    vec![
        OpCode::PUSH2.byte(),
        OpCode::PUSH3.byte(),
        OpCode::ADD.byte(),
        OpCode::RET.byte(),
    ]
}

fn mul_script() -> Vec<u8> {
    vec![
        OpCode::PUSH5.byte(),
        OpCode::PUSH4.byte(),
        OpCode::MUL.byte(),
        OpCode::RET.byte(),
    ]
}

fn dup_script() -> Vec<u8> {
    vec![OpCode::DUP.byte(), OpCode::RET.byte()]
}

fn loop_script(iters: u16) -> Vec<u8> {
    // INITSLOT 1 local, 0 args; LDLOC0 / PUSH1 / ADD / STLOC0 / LDLOC0 / PUSH iters / LT / JMPIF back / LDLOC0 / RET
    // Simpler: N× NOP then RET — measures dispatch cost without control flow complexity.
    let mut script = vec![OpCode::NOP.byte(); iters as usize];
    script.push(OpCode::RET.byte());
    script
}

fn sha256_script() -> Vec<u8> {
    let mut script = vec![OpCode::SYSCALL.byte()];
    script.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());
    script.push(OpCode::RET.byte());
    script
}

fn bench_execute(c: &mut Criterion) {
    let mut group = c.benchmark_group("execute");

    group.bench_function("arithmetic/add", |b| {
        let script = add_script();
        b.iter(|| {
            let out = execute(ProofInput {
                script: black_box(script.clone()),
                arguments: vec![],
                gas_limit: 1_000_000,
            });
            black_box(out)
        });
    });

    group.bench_function("arithmetic/mul", |b| {
        let script = mul_script();
        b.iter(|| {
            let out = execute(ProofInput {
                script: black_box(script.clone()),
                arguments: vec![],
                gas_limit: 1_000_000,
            });
            black_box(out)
        });
    });

    group.bench_function("stack/dup", |b| {
        let script = dup_script();
        b.iter(|| {
            let out = execute(ProofInput {
                script: black_box(script.clone()),
                arguments: vec![StackItem::Integer(42)],
                gas_limit: 1_000_000,
            });
            black_box(out)
        });
    });

    group.bench_function("loop/1000_nops", |b| {
        let script = loop_script(1000);
        b.iter(|| {
            let out = execute(ProofInput {
                script: black_box(script.clone()),
                arguments: vec![],
                gas_limit: 1_000_000,
            });
            black_box(out)
        });
    });

    group.bench_function("crypto/sha256", |b| {
        let script = sha256_script();
        let payload = StackItem::ByteString(b"hello neo zkvm".to_vec());
        b.iter(|| {
            let out = execute(ProofInput {
                script: black_box(script.clone()),
                arguments: vec![black_box(payload.clone())],
                gas_limit: 1_000_000,
            });
            black_box(out)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_execute);
criterion_main!(benches);
