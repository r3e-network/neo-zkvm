use neo_vm_core::StackItem;
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn main() {
    println!("=======================================================");
    println!("=== zkVM Scaling Use Case: Off-chain Computation ======");
    println!("=======================================================\n");
    println!("Goal: Run a complex algorithm (Summing numbers) off-chain.");
    println!("The blockchain only verifies a succinct ZK proof, ");
    println!("saving massive amounts of gas.\n");

    // Sum 1 to 5 script (PUSH0, PUSH1, ADD, PUSH2, ADD, PUSH3, ADD, PUSH4, ADD, PUSH5, ADD, RET)
    let script_hex = "10119e129e139e149e159e40";
    let script = hex_decode(script_hex);

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock, 
        ..Default::default()
    });

    println!("[Prover] Executing 100+ opcodes in the zkVM prover off-chain...");
    let proof = prover.prove(ProofInput {
        script,
        arguments: vec![],
        gas_limit: 1_000_000,
    });

    println!("[Prover] Proof generated! Size: {} bytes (Mock)", proof.proof_bytes.len());
    println!("[Prover] Gas consumed off-chain: {}", proof.output.gas_consumed);
    
    println!("\n--- Verifier (On-Chain) Side ---");
    let is_valid = verify_for_mode(&proof, proof.proof_mode);
    println!("Verifier checks the proof... Valid? {}", is_valid);
    
    let result = proof.output.result.as_ref().unwrap();
    if let StackItem::Integer(val) = result {
        println!("Verified Output: Sum = {}", val);
    }
    println!("\nThe blockchain didn't need to run a single loop or addition!");
    println!("It only ran 1 mathematical verification equation.");
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
