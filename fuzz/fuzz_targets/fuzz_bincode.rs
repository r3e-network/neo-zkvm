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
    let slice = if data.len() > 64 * 1024 {
        &data[..64 * 1024]
    } else {
        data
    };

    let _ = bincode_deserialize::<NeoProof>(slice);
    let _ = bincode_deserialize::<ProofInput>(slice);
    let _ = bincode_deserialize::<ProofOutput>(slice);
    let _ = bincode_deserialize::<PublicInputs>(slice);
    let _ = bincode_deserialize::<MockProof>(slice);
    let _ = bincode_deserialize::<StackItem>(slice);
    let _ = bincode_deserialize::<ProofMode>(slice);
    let _ = deserialize_neoproof(slice);

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
