use neo_vm_guest::{OpCode, ProofInput, StackItem};
use neo_zkvm_prover::{NeoProof, NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn prove_square_of_secret(secret: i64) -> NeoProof {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });

    prover.prove(ProofInput {
        script: vec![OpCode::DUP.byte(), OpCode::MUL.byte(), OpCode::RET.byte()],
        arguments: vec![StackItem::Integer(secret)],
        gas_limit: 1_000_000,
    })
}

fn read_integer_result(proof: &NeoProof) -> Option<i128> {
    proof.output.result.as_ref().and_then(|item| item.to_i128())
}

fn main() {
    let secret_a = 7;
    let secret_b = 8;

    let proof_a = prove_square_of_secret(secret_a);
    let proof_b = prove_square_of_secret(secret_b);

    println!("=== Private Inputs Example ===");
    println!("Result(secret_a^2): {:?}", read_integer_result(&proof_a));
    println!("Result(secret_b^2): {:?}", read_integer_result(&proof_b));
    println!(
        "Proof A verifies: {}",
        verify_for_mode(&proof_a, ProofMode::Mock)
    );
    println!(
        "Proof B verifies: {}",
        verify_for_mode(&proof_b, ProofMode::Mock)
    );
    println!(
        "Input hash A: 0x{}",
        hex::encode(proof_a.public_inputs.input_hash)
    );
    println!(
        "Input hash B: 0x{}",
        hex::encode(proof_b.public_inputs.input_hash)
    );
    println!(
        "Different private inputs produce different commitments: {}",
        proof_a.public_inputs.input_hash != proof_b.public_inputs.input_hash
    );

    assert!(verify_for_mode(&proof_a, ProofMode::Mock));
    assert!(verify_for_mode(&proof_b, ProofMode::Mock));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_input_proof_verifies_and_returns_square() {
        let proof = prove_square_of_secret(7);

        assert!(verify_for_mode(&proof, ProofMode::Mock));
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
        assert!(verify_for_mode(&proof_a, ProofMode::Mock));
        assert!(verify_for_mode(&proof_b, ProofMode::Mock));
    }
}
