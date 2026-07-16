//! Fuzz structured VM execution (balanced fragments + optional arguments).
//!
//! Goal: no panics / unbounded hang for representative Neo opcode sequences.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{execute, ProofInput};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let script = common::build_structured_script(data, 256);
    let arguments = common::arguments_from_tail(data, 256);

    let _ = execute(ProofInput {
        script,
        arguments,
        gas_limit: common::DEFAULT_FUZZ_GAS,
    });
});
