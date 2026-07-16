//! Shared helpers for Neo zkVM examples (Ethereum-style patterns).
//!
//! Included via `#[path]` into multiple binaries; not every helper is used by
//! every example.

#![allow(dead_code)]

use neo_vm_guest::{OpCode, ProofInput, StackItem};
use neo_zkvm_prover::{NeoProof, NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

/// Mock prover for local demos. Production must use Sp1/Plonk/Groth16.
pub fn mock_prover() -> NeoProver {
    NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        deterministic_mock_timestamp: Some(1),
        ..ProverConfig::default()
    })
}

pub fn prove_mock(script: Vec<u8>, arguments: Vec<StackItem>) -> NeoProof {
    mock_prover().prove(ProofInput {
        script,
        arguments,
        gas_limit: 1_000_000,
    })
}

pub fn verify_mock(proof: &NeoProof) -> bool {
    verify_for_mode(proof, ProofMode::Mock)
}

pub fn result_i128(proof: &NeoProof) -> Option<i128> {
    proof.output.result.as_ref().and_then(|i| i.to_i128())
}

pub fn result_bytes(proof: &NeoProof) -> Option<&[u8]> {
    match proof.output.result.as_ref() {
        Some(StackItem::ByteString(b)) | Some(StackItem::Buffer(_, b)) => Some(b.as_slice()),
        _ => None,
    }
}

/// Encode a small positive integer as Neo push + return pattern helpers.
pub fn push_small(n: u8) -> u8 {
    // PUSH0..PUSH16 are 0x10..0x20
    assert!(n <= 16);
    0x10 + n
}

pub fn script_from_opcodes(ops: &[OpCode]) -> Vec<u8> {
    ops.iter().map(|o| o.byte()).collect()
}

/// INITSLOT locals=0, args=n then body opcodes ending with RET.
pub fn script_with_args(arg_count: u8, body: &[u8]) -> Vec<u8> {
    let mut s = vec![OpCode::INITSLOT.byte(), 0x00, arg_count];
    s.extend_from_slice(body);
    s
}

/// Build argument vector for NeoVM `INITSLOT` slots.
///
/// Neo pops the evaluation stack **top-first** into `arg0..argN`. The guest
/// supplies the initial stack in vec order where the **last** element is top.
/// Therefore `args_in_ldarg_order[0]` (LDARG0) must be the **last** pushed
/// element: we reverse here so callers can write natural `[arg0, arg1, ...]`.
pub fn neo_call_args(args_in_ldarg_order: Vec<StackItem>) -> Vec<StackItem> {
    let mut v = args_in_ldarg_order;
    v.reverse();
    v
}

pub fn banner(title: &str) {
    println!("=======================================================");
    println!("=== {title}");
    println!("=======================================================");
    println!("NOTE: Uses ProofMode::Mock for local demos (not ZK-secure).");
    println!("Production: pin Sp1/Plonk/Groth16 and bind public claims.\n");
}
