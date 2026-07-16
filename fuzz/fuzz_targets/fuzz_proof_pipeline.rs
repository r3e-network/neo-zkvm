//! Fuzz mock prove → verify → serialize → deserialize → re-verify.
//!
//! Catches panics in prover/verifier and invariant breaks under adversarial scripts.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::{
    ProofMode, bincode_deserialize, bincode_serialize, deserialize_neoproof, execute,
};
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Keep structured scripts + gas modest so long campaigns stay under RSS caps.
    let script = common::build_structured_script(data, 32);
    let arguments = common::arguments_from_tail(data, 32);
    // Cap gas harder than default clamp_gas (100k) for prove+serialize path.
    let gas_limit = (u64::from(data[0]).saturating_mul(50).max(1)).min(10_000);

    // Host execute (no proof) — must not panic.
    let _ = execute(neo_vm_guest::ProofInput {
        script: script.clone(),
        arguments: arguments.clone(),
        gas_limit,
    });

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        deterministic_mock_timestamp: Some(42),
        ..ProverConfig::default()
    });

    let proof = prover.prove(neo_vm_guest::ProofInput {
        script,
        arguments,
        gas_limit,
    });

    // Mode-pinned verification (may fail for faulting scripts — OK).
    let _ = verify_for_mode(&proof, ProofMode::Mock);
    let _ = prover.verify(&proof);

    // Serialization round-trip of the NeoProof envelope.
    if let Ok(bytes) = bincode_serialize(&proof) {
        if let Ok(decoded) = bincode_deserialize::<neo_vm_guest::NeoProof>(&bytes) {
            let _ = verify_for_mode(&decoded, ProofMode::Mock);
        }
        // Legacy-compatible entry point
        let _ = deserialize_neoproof(&bytes);
    }

    // Tamper resistance: flip a public-input byte and ensure verification fails
    // when the original proof was valid.
    if proof.output.state == 0 && verify_for_mode(&proof, ProofMode::Mock) {
        let mut tampered = proof.clone();
        tampered.public_inputs.script_hash[0] ^= 0xFF;
        assert!(
            !verify_for_mode(&tampered, ProofMode::Mock),
            "tampered public inputs must not verify"
        );
        let mut out_tampered = proof;
        if let Some(ref mut item) = out_tampered.output.result {
            *item = neo_vm_guest::StackItem::Integer(0xDEAD);
        } else {
            out_tampered.output.result = Some(neo_vm_guest::StackItem::Integer(0xDEAD));
        }
        // May fail output_matches check even if result was None.
        let _ = verify_for_mode(&out_tampered, ProofMode::Mock);
    }
});
