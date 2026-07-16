//! Shared helpers for neo-zkvm fuzz targets.
//!
//! Generators aim for two goals simultaneously:
//! 1. **Validity bias** — emit sequences that often execute without immediate stack faults
//! 2. **Adversarial coverage** — still allow junk/raw bytes so panics and OOM paths are hit
//!
//! Each fuzz binary includes this module independently, so not every helper is
//! used by every target — allow dead_code at the module level.

#![allow(dead_code)]

use neo_vm_guest::{OpCode, StackItem, interop_hash};

/// Bounded, stack-balanced opcode fragments (used by structured fuzzers).
pub fn append_bounded_neo_vm_sequence(script: &mut Vec<u8>, byte: u8) {
    match byte % 24 {
        0 => script.push(OpCode::PUSH0.byte()),
        1 => script.extend_from_slice(&[OpCode::PUSH1.byte(), OpCode::DROP.byte()]),
        2 => script.extend_from_slice(&[
            OpCode::PUSH2.byte(),
            OpCode::PUSH3.byte(),
            OpCode::ADD.byte(),
            OpCode::DROP.byte(),
        ]),
        3 => script.extend_from_slice(&[
            OpCode::PUSH3.byte(),
            OpCode::PUSH1.byte(),
            OpCode::SUB.byte(),
            OpCode::DROP.byte(),
        ]),
        4 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::PUSH2.byte(),
            OpCode::MUL.byte(),
            OpCode::DROP.byte(),
        ]),
        5 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::PUSH1.byte(),
            OpCode::NUMEQUAL.byte(),
            OpCode::DROP.byte(),
        ]),
        6 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::NOT.byte(),
            OpCode::DROP.byte(),
        ]),
        7 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::DUP.byte(),
            OpCode::DROP.byte(),
            OpCode::DROP.byte(),
        ]),
        8 => script.extend_from_slice(&[
            OpCode::PUSH1.byte(),
            OpCode::PUSH2.byte(),
            OpCode::SWAP.byte(),
            OpCode::DROP.byte(),
            OpCode::DROP.byte(),
        ]),
        9 => script.extend_from_slice(&[OpCode::PUSH1.byte(), OpCode::DROP.byte()]),
        10 => script.extend_from_slice(&[OpCode::PUSH2.byte(), OpCode::DROP.byte()]),
        11 => script.push(OpCode::NOP.byte()),
        // Comparisons / bitwise
        12 => script.extend_from_slice(&[
            OpCode::PUSH5.byte(),
            OpCode::PUSH3.byte(),
            OpCode::LT.byte(),
            OpCode::DROP.byte(),
        ]),
        13 => script.extend_from_slice(&[
            OpCode::PUSH7.byte(),
            OpCode::PUSH3.byte(),
            OpCode::AND.byte(),
            OpCode::DROP.byte(),
        ]),
        14 => script.extend_from_slice(&[
            OpCode::PUSH4.byte(),
            OpCode::PUSH1.byte(),
            OpCode::SHL.byte(),
            OpCode::DROP.byte(),
        ]),
        // Arrays
        15 => script.extend_from_slice(&[
            OpCode::PUSH3.byte(),
            OpCode::NEWARRAY.byte(),
            OpCode::SIZE.byte(),
            OpCode::DROP.byte(),
        ]),
        // Deterministic crypto adapters (need ByteString arg on stack)
        16 => {
            // empty ByteString via PUSHDATA1 0, then SYSCALL SHA256, DROP result
            script.extend_from_slice(&[OpCode::PUSHDATA1.byte(), 0x00]);
            script.push(OpCode::SYSCALL.byte());
            script.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());
            script.push(OpCode::DROP.byte());
        }
        17 => {
            script.extend_from_slice(&[OpCode::PUSHDATA1.byte(), 0x01, b'a']);
            script.push(OpCode::SYSCALL.byte());
            script.extend_from_slice(&interop_hash("System.Crypto.RIPEMD160").to_le_bytes());
            script.push(OpCode::DROP.byte());
        }
        18 => {
            script.extend_from_slice(&[OpCode::PUSHDATA1.byte(), 0x01, b'b']);
            script.push(OpCode::SYSCALL.byte());
            script.extend_from_slice(&interop_hash("System.Crypto.Hash160").to_le_bytes());
            script.push(OpCode::DROP.byte());
        }
        19 => {
            script.extend_from_slice(&[OpCode::PUSHDATA1.byte(), 0x01, b'c']);
            script.push(OpCode::SYSCALL.byte());
            script.extend_from_slice(&interop_hash("System.Crypto.Hash256").to_le_bytes());
            script.push(OpCode::DROP.byte());
        }
        // Short forward JMP over a NOP
        20 => script.extend_from_slice(&[OpCode::JMP.byte(), 0x02, OpCode::NOP.byte()]),
        // DIV that may fault (div by zero when top is 0) — still must not panic
        21 => script.extend_from_slice(&[
            OpCode::PUSH5.byte(),
            OpCode::PUSH0.byte(),
            OpCode::DIV.byte(),
        ]),
        // Stack underflow attempt
        22 => script.push(OpCode::DROP.byte()),
        // Invalid / reserved byte mixed in
        _ => script.push(0xFF),
    }
}

/// Build a script from structured bytes and always terminate with RET.
pub fn build_structured_script(seed: &[u8], max_fragments: usize) -> Vec<u8> {
    let mut script = Vec::with_capacity(max_fragments.saturating_mul(8).saturating_add(1));
    for byte in seed.iter().take(max_fragments) {
        append_bounded_neo_vm_sequence(&mut script, *byte);
    }
    script.push(OpCode::RET.byte());
    script
}

/// Cap raw bytecode size for adversarial scripts.
pub const MAX_RAW_SCRIPT: usize = 512;
pub const MAX_ARG_BYTES: usize = 2048;
pub const DEFAULT_FUZZ_GAS: u64 = 50_000;

pub fn clamp_gas(raw: u64) -> u64 {
    // Always at least 1 so zero-gas paths are hit via dedicated cases, not by accident of %.
    (raw % 100_000).max(1)
}

pub fn arguments_from_tail(data: &[u8], start: usize) -> Vec<StackItem> {
    if data.len() <= start {
        return vec![];
    }
    let tail = &data[start..];
    if tail.is_empty() {
        return vec![];
    }
    // Split tail into up to 4 ByteString args of limited size.
    let mut args = Vec::new();
    let chunk = (tail.len() / 4).max(1).min(MAX_ARG_BYTES);
    for piece in tail.chunks(chunk).take(4) {
        if !piece.is_empty() {
            args.push(StackItem::ByteString(piece.to_vec()));
        }
    }
    args
}
