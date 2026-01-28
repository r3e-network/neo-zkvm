//! Neo zkVM CLI

use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify;

fn main() {
    println!("Neo zkVM v0.1.0");

    // Example: prove 2 + 3
    let script = vec![
        0x12, // PUSH2
        0x13, // PUSH3
        0x9E, // ADD
        0x40, // RET
    ];

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    };

    let prover = NeoProver::new(ProverConfig::default());
    let proof = prover.prove(input);

    println!("Result: {:?}", proof.output.result);
    println!("Verified: {}", verify(&proof));
}
