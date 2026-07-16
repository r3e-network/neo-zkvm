//! Fuzz deserialization of proof envelopes and related types.
//!
//! Guarantees: never panic; reject oversized/malformed payloads cleanly.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{
    MockProof, NeoProof, ProofInput, ProofMode, ProofOutput, PublicInputs, StackItem,
    bincode_deserialize, bincode_serialize, deserialize_neoproof, execute,
};

fuzz_target!(|data: &[u8]| {
    // Cap input so we exercise the bincode limit path without huge allocations.
    // Keep this modest: adversarial length prefixes still go through BINCODE_LIMIT,
    // but smaller inputs reduce peak RSS under long high-throughput campaigns.
    let slice = if data.len() > 8 * 1024 {
        &data[..8 * 1024]
    } else {
        data
    };

    // Rotate which type we stress so one iteration does less peak work.
    match data.first().copied().unwrap_or(0) % 8 {
        0 => {
            let _ = bincode_deserialize::<NeoProof>(slice);
        }
        1 => {
            let _ = bincode_deserialize::<ProofInput>(slice);
        }
        2 => {
            let _ = bincode_deserialize::<ProofOutput>(slice);
        }
        3 => {
            let _ = bincode_deserialize::<PublicInputs>(slice);
        }
        4 => {
            let _ = bincode_deserialize::<MockProof>(slice);
        }
        5 => {
            let _ = bincode_deserialize::<StackItem>(slice);
        }
        6 => {
            let _ = bincode_deserialize::<ProofMode>(slice);
        }
        _ => {
            let _ = deserialize_neoproof(slice);
        }
    }

    // Round-trip of freshly executed small scripts under serialization.
    if !data.is_empty() {
        let script = common::build_structured_script(data, 16);
        let output = execute(ProofInput {
            script: script.clone(),
            arguments: vec![],
            gas_limit: 1_000,
        });
        if let Ok(bytes) = bincode_serialize(&output) {
            let _ = bincode_deserialize::<ProofOutput>(&bytes);
        }
        let input = ProofInput {
            script,
            arguments: common::arguments_from_tail(data, 16),
            gas_limit: 1_000,
        };
        if let Ok(bytes) = bincode_serialize(&input) {
            let _ = bincode_deserialize::<ProofInput>(&bytes);
        }
    }
});
