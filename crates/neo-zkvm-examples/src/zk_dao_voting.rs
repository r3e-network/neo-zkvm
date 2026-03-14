use neo_vm_core::StackItem;
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn main() {
    println!("=======================================================");
    println!("=== zkVM Privacy: Anonymous DAO Voting     ============");
    println!("=======================================================\n");
    println!("Problem: On-chain voting is public. Whales can influence");
    println!("voters, and privacy is non-existent.");
    println!("Solution: Users generate a ZK proof locally showing their");
    println!("vote is valid (Candidate 1 or 2) without revealing WHICH");
    println!("candidate they picked. The contract tallies ZK proofs.\n");

    // We want to prove that the private vote is either 1 or 2.
    // Script Logic (Validation):
    // 1. DUP the vote
    // 2. Push 1, NUMEQUAL. If true, jump to success.
    // 3. DUP the vote (again, if 1 failed)
    // 4. Push 2, NUMEQUAL. If true, jump to success.
    // 5. If neither, return 0 (Failure)
    // 6. Success: return 1
    // Bytecode assembled from:
    // DUP, PUSH1, NUMEQUAL, JMPIF 6, DUP, PUSH2, NUMEQUAL, JMPIF 3, PUSH0, RET, PUSH1, RET
    let script_hex = "4a11b3240a4a12b32405451040451140";
    let script = hex_decode(script_hex);

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock, 
        ..Default::default()
    });

    let secret_vote = 2; // User votes for candidate 2

    println!("[Prover] Executing DAO vote validation locally...");
    println!("[Prover] Private Vote Choice: {}", secret_vote);

    let proof = prover.prove(ProofInput {
        script,
        arguments: vec![StackItem::Integer(secret_vote)],
        gas_limit: 1_000_000,
    });

    println!("[Prover] Vote ZK-Proof generated! Size: {} bytes", proof.proof_bytes.len());
    
    println!("\n--- DAO Smart Contract (On-Chain) Side ---");
    let is_valid = verify_for_mode(&proof, proof.proof_mode);
    println!("DAO verifies the proof... Is the vote mathematically valid? {}", is_valid);
    
    let result = proof.output.result.as_ref().unwrap();
    if let StackItem::Integer(val) = result {
        println!("Execution Output: {}", val); // 1 = valid vote, 0 = invalid
        if *val == 1 {
            println!("Result: The vote is VALID. The DAO increments the tally using");
            println!("homomorphic encryption or nullifiers combined with this ZK-proof,");
            println!("meaning the vote is counted but the choice remains completely secret!");
        }
    }
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
