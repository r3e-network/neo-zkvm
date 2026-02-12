use neo_vm_core::StackItem;
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProof, NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify;

fn prove_square_of_secret(secret: i128) -> NeoProof {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });

    prover.prove(ProofInput {
        script: vec![0x4A, 0xA0, 0x40],
        arguments: vec![StackItem::Integer(secret)],
        gas_limit: 1_000_000,
    })
}

fn read_integer_result(proof: &NeoProof) -> Option<i128> {
    proof
        .output
        .result
        .as_ref()
        .and_then(|item| item.to_integer())
}

fn main() {
    let secret_a = 7;
    let secret_b = 8;

    let proof_a = prove_square_of_secret(secret_a);
    let proof_b = prove_square_of_secret(secret_b);

    println!("=== Private Inputs Example ===");
    println!("Result(secret_a^2): {:?}", read_integer_result(&proof_a));
    println!("Result(secret_b^2): {:?}", read_integer_result(&proof_b));
    println!("Proof A verifies: {}", verify(&proof_a));
    println!("Proof B verifies: {}", verify(&proof_b));
    println!(
        "Input hash A: 0x{}",
        hex_encode(&proof_a.public_inputs.input_hash)
    );
    println!(
        "Input hash B: 0x{}",
        hex_encode(&proof_b.public_inputs.input_hash)
    );
    println!(
        "Different private inputs produce different commitments: {}",
        proof_a.public_inputs.input_hash != proof_b.public_inputs.input_hash
    );

    assert!(verify(&proof_a));
    assert!(verify(&proof_b));
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_input_proof_verifies_and_returns_square() {
        let proof = prove_square_of_secret(7);

        assert!(verify(&proof));
        assert_eq!(read_integer_result(&proof), Some(49));
    }

    #[test]
    fn test_different_private_inputs_produce_different_input_hashes() {
        let proof_a = prove_square_of_secret(7);
        let proof_b = prove_square_of_secret(8);

        assert_ne!(
            proof_a.public_inputs.input_hash,
            proof_b.public_inputs.input_hash
        );
        assert!(verify(&proof_a));
        assert!(verify(&proof_b));
    }
}
