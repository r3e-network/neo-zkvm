//! Fuzz raw/unstructured bytecode under a tight gas budget.
//!
//! This is the adversarial path: invalid opcodes, truncated operands, deep
//! loops — the guest must never panic or hang past the gas meter.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, ProofInput};

fuzz_target!(|data: &[u8]| {
    // Mode byte selects gas/arg variants for broader coverage.
    let mode = data.first().copied().unwrap_or(0);
    let body = if data.len() > 1 { &data[1..] } else { &[] };

    let script_len = body.len().min(common::MAX_RAW_SCRIPT);
    let mut script = body[..script_len].to_vec();
    // Always append RET so pure-fallthrough scripts can halt if they get that far.
    if script.last() != Some(&0x40) {
        script.push(0x40); // RET
    }

    let gas_limit = match mode % 5 {
        0 => 1, // minimal budget
        1 => 10,
        2 => 100,
        3 => 1_000,
        _ => common::DEFAULT_FUZZ_GAS,
    };

    let arguments = if mode % 3 == 0 {
        common::arguments_from_tail(body, script_len)
    } else {
        vec![]
    };

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit,
    });
});
