use neo_vm_core::StackItem;
use neo_vm_guest::ProofInput;
use neo_zkvm_prover::{NeoProof, NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_detailed;

fn generate_valid_mock_proof() -> NeoProof {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });

    prover.prove(ProofInput {
        script: vec![0x12, 0x13, 0x9E, 0x40],
        arguments: vec![],
        gas_limit: 1_000_000,
    })
}

fn main() {
    let proof = generate_valid_mock_proof();

    println!("=== Tamper Resistance Example ===");
    println!("Original proof valid: {}", verify_detailed(&proof).valid);

    let mut output_tampered = proof.clone();
    output_tampered.output.result = Some(StackItem::Integer(999));
    let output_result = verify_detailed(&output_tampered);
    println!("Output tamper valid: {}", output_result.valid);
    if let Some(error) = output_result.error {
        println!("Output tamper error: {}", error);
    }

    let mut hash_tampered = proof.clone();
    hash_tampered.public_inputs.output_hash[0] ^= 0x01;
    let hash_result = verify_detailed(&hash_tampered);
    println!("Public-input tamper valid: {}", hash_result.valid);
    if let Some(error) = hash_result.error {
        println!("Public-input tamper error: {}", error);
    }

    let mut version_tampered = proof.clone();
    version_tampered.proof_format_version = version_tampered.proof_format_version.saturating_add(1);
    let version_result = verify_detailed(&version_tampered);
    println!("Version tamper valid: {}", version_result.valid);
    if let Some(error) = version_result.error {
        println!("Version tamper error: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_tampering_is_rejected() {
        let proof = generate_valid_mock_proof();

        let mut tampered = proof.clone();
        tampered.output.result = Some(StackItem::Integer(999));

        let result = verify_detailed(&tampered);
        assert!(!result.valid);
    }

    #[test]
    fn test_proof_format_tampering_is_rejected() {
        let proof = generate_valid_mock_proof();

        let mut tampered = proof.clone();
        tampered.proof_format_version = tampered.proof_format_version.saturating_add(1);

        let result = verify_detailed(&tampered);
        assert!(!result.valid);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Unsupported proof format version"));
    }
}
