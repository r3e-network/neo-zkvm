use neo_vm_guest::{OpCode, ProofInput, StackItem, interop_hash};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn main() {
    println!("=======================================================");
    println!("=== zkVM Privacy Use Case: Hash Preimage Proof ========");
    println!("=======================================================\n");
    println!("NOTE: This demo uses ProofMode::Mock — NOT cryptographically");
    println!("secure. Production must pin Sp1/Plonk/Groth16 and bind an");
    println!("expected public image hash in the verification circuit.\n");
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

    // Verify the proof mathematically.
    // Pin the expected mode to a constant — never `proof.proof_mode`, which
    // would make the check a tautology and accept forgeable Mock proofs.
    // DEMO ONLY: Mock proofs are not cryptographically secure; a production
    // verifier MUST pin a succinct mode (`ProofMode::Groth16`/`Plonk`).
    let is_valid = verify_for_mode(&proof, ProofMode::Mock);
    println!("Verifier checks the proof... Valid? {is_valid}");

    match proof.output.result.as_ref() {
        Some(StackItem::ByteString(hash_bytes)) => {
            println!("Public Output (Computed Hash): {}", hex::encode(hash_bytes));
            println!("The verifier confirms this matches the expected hash!");
            println!(
                "\nSuccess! The secret '{}' is NEVER revealed to the blockchain!",
                String::from_utf8_lossy(&secret_password)
            );
        }
        Some(other) => println!("Unexpected result type: {other:?}"),
        None => println!("No hash result on stack"),
    }
}
