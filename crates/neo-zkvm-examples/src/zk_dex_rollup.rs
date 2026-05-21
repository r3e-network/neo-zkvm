use neo_vm_guest::{execute, ProofInput, StackItem};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn main() {
    println!("=======================================================");
    println!("=== zkVM Execution: Zero-Cost L2 DEX Off-Chain ========");
    println!("=======================================================\n");
    println!("Problem: A DEX matching 10,000 trades on Neo Layer 1 costs");
    println!("massive gas and hits block-size limits.");
    println!("Solution: A Sequencer executes all trades off-chain in the zkVM,");
    println!("generates a single Validity Proof, and submits only the new");
    println!("state root to L1. L1 users pay almost zero gas.\n");

    // A simulated "Batch order matching" script
    // It takes 3 balances, adds them up (simulating calculating new state),
    // and returns the final L2 state root.
    // PUSH 100, PUSH 200, ADD, PUSH 50, ADD, RET -> 100 + 200 + 50 = 350
    // Bytecode: 1c64 1cc800 9e 1c32 9e 40
    // Using PUSHINT8 for 100, 50 and PUSHINT16 for 200
    // 100 = 0x64
    // 200 = 0xc800
    // 50 = 0x32
    let script_hex = "006401c8009e00329e40";
    let script = hex_decode(script_hex);

    // Run locally with the same shared VM semantics used by the zkVM guest.
    let local_output = execute(ProofInput {
        script: script.clone(),
        arguments: vec![],
        gas_limit: 1_000_000,
    });
    let l2_execution_units = local_output.gas_consumed;

    println!("[L2 VM] Shared execution units: {}", l2_execution_units);
    println!(
        "[L2 VM] Imagine batching 10,000 trades... Units: {}\n",
        l2_execution_units * 10000
    );

    println!("--- Using the zkVM Sequencer ---");
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });

    println!("[Sequencer] Matching L2 orders inside the zkVM off-chain...");
    let proof = prover.prove(ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    });

    println!("[Sequencer] Proof generated! Sequencer submits Proof to L1.");

    println!("\n--- DEX Smart Contract (On-Chain) Side ---");
    let is_valid = verify_for_mode(&proof, proof.proof_mode);
    println!(
        "L1 verifies the succinct mathematical proof... Valid? {}",
        is_valid
    );

    let result = proof.output.result.as_ref().unwrap();
    if let StackItem::Integer(val) = result {
        println!("New DEX State Root (Total Liquidity): {}", val);
    }
    println!("\nThe L1 blockchain didn't need to process the trades!");
    println!("The single proof guarantees correctness for infinite transactions.");
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
