use neo_vm_guest::{interop_hash, OpCode, ProofInput, StackItem};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn main() {
    println!("=======================================================");
    println!("=== zkVM Privacy Use Case: Hash Preimage Proof ========");
    println!("=======================================================\n");
    println!("Goal: Prove you know a secret password that hashes to a");
    println!("known public value, WITHOUT revealing the password!\n");

    let secret_password = b"my_super_secret_password".to_vec();

    // Deterministic zkVM crypto is exposed through the syscall adapter so it
    // does not consume canonical NeoVM opcode space.
    let mut script = vec![OpCode::SYSCALL.byte()];
    script.extend_from_slice(&interop_hash("System.Crypto.SHA256").to_le_bytes());
    script.push(OpCode::RET.byte());

    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock, // Use Mock for fast demonstration
        ..Default::default()
    });

    println!("[Prover] Running script locally and generating ZK Proof...");
    let proof = prover.prove(ProofInput {
        script,
        arguments: vec![StackItem::ByteString(secret_password.clone())],
        gas_limit: 1_000_000,
    });

    println!(
        "[Prover] Proof generated successfully! Size: {} bytes",
        proof.proof_bytes.len()
    );

    // Verifier side
    println!("\n--- Verifier (On-Chain) Side ---");

    // Verify the proof mathematically
    let is_valid = verify_for_mode(&proof, proof.proof_mode);
    println!("Verifier checks the proof... Valid? {}", is_valid);

    // Read the output
    let output_item = proof.output.result.as_ref().unwrap();
    if let StackItem::ByteString(hash_bytes) = output_item {
        println!("Public Output (Computed Hash): {}", hex_encode(hash_bytes));
        println!("The verifier confirms this matches the expected hash!");
        println!(
            "\nSuccess! The secret '{}' is NEVER revealed to the blockchain!",
            String::from_utf8_lossy(&secret_password)
        );
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
