//! End-to-end **settlement path** for current Neo N3 (contracts only).
//!
//! Off-chain:
//!   1. Prove NeoVM script (Mock here for CI; production uses SP1 Groth16)
//!   2. Verify proof mode-pinned
//!   3. N-of-M attestors sign canonical digest
//!
//! On-chain (see contracts/neo-n3/NeoZkAttestation):
//!   4. Contract checks ECDSA threshold + public claim, then settles
//!
//! This binary demonstrates steps 1–3 with a demo 2-of-3 committee.

#[path = "common.rs"]
mod common;

use neo_vm_guest::{OpCode, ProofMode, StackItem};
use neo_zkvm_attestation::{
    AttestationClaim, AttestorKeypair, ProofModeCode, app_claim_hash, attest_bundle,
    attestation_digest, verify_threshold,
};
use neo_zkvm_prover::{NeoProver, ProverConfig};
use neo_zkvm_verifier::verify_for_mode;

fn factors_script() -> Vec<u8> {
    common::script_with_args(
        2,
        &[
            OpCode::LDARG0.byte(),
            OpCode::LDARG1.byte(),
            OpCode::MUL.byte(),
            OpCode::RET.byte(),
        ],
    )
}

fn main() {
    common::banner("zk_attestation_settlement — SP1 path + N-of-M Neo settlement");

    // --- 1) Prove NeoVM (Mock for local/CI; use Groth16 in production) ---
    let p = 13i64;
    let q = 17i64;
    let n = p * q;
    let script = factors_script();
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        deterministic_mock_timestamp: Some(1),
        ..ProverConfig::default()
    });
    let proof = prover.prove(neo_vm_guest::ProofInput {
        script,
        arguments: common::neo_call_args(vec![StackItem::Integer(p), StackItem::Integer(q)]),
        gas_limit: 1_000_000,
    });

    assert!(verify_for_mode(&proof, ProofMode::Mock));
    assert_eq!(common::result_i128(&proof), Some(n as i128));
    println!("[1] NeoVM proof verified off-chain (Mock demo). product={n}");

    // --- 2) Build attestation claim ---
    // Production: program_id = hash of SP1 verifying key; proof_mode = Groth16.
    let program_id = neo_vm_guest::hash_data(b"neo-zkvm-program:demo-v1");
    let claim = AttestationClaim {
        program_id,
        // Demo allows Mock only with allow_unsafe_mode=true below.
        proof_mode: ProofModeCode::from(ProofMode::Mock),
        public_inputs: proof.public_inputs.clone(),
        app_claim_hash: app_claim_hash(&n.to_le_bytes()),
        network_magic: 0x334f_454e,
        nonce: neo_vm_guest::hash_data(b"demo-nonce-1"),
    };
    let digest = attestation_digest(&claim);
    println!("[2] Attestation digest: {}", hex::encode(digest));

    // --- 3) 2-of-3 attestor committee ---
    let a1 = AttestorKeypair::generate();
    let a2 = AttestorKeypair::generate();
    let a3 = AttestorKeypair::generate();
    let bundle = attest_bundle(claim.clone(), &[&a1, &a2], true /* demo Mock */).unwrap();
    let authorized = vec![
        a1.public_key_uncompressed(),
        a2.public_key_uncompressed(),
        a3.public_key_uncompressed(),
    ];
    verify_threshold(&bundle.claim, &bundle.signatures, &authorized, 2, true).unwrap();
    println!(
        "[3] N-of-M ECDSA OK (2-of-3). signatures={}",
        bundle.signatures.len()
    );

    println!("\nSubmit AttestationBundle to Neo N3 contract NeoZkAttestation.Submit:");
    println!("  program_id     = {}", hex::encode(program_id));
    println!("  app_claim_hash = {}", hex::encode(claim.app_claim_hash));
    println!("  nonce          = {}", hex::encode(claim.nonce));
    println!("  threshold      = 2 / 3 attestors");
    println!("\nContract then: VerifyWithECDsa × N, check claim, settle app logic.");
    println!("Production: proof_mode=Groth16, allow_unsafe_mode=false, real SP1 verify first.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use neo_zkvm_attestation::ProofModeCode;

    #[test]
    fn settlement_demo_threshold_holds() {
        let k1 = AttestorKeypair::generate();
        let k2 = AttestorKeypair::generate();
        let claim = AttestationClaim {
            program_id: [1u8; 32],
            proof_mode: ProofModeCode::Groth16,
            public_inputs: neo_vm_guest::PublicInputs {
                script_hash: [2u8; 32],
                input_hash: [3u8; 32],
                output_hash: [4u8; 32],
                gas_consumed: 10,
                execution_success: true,
            },
            app_claim_hash: app_claim_hash(b"claim"),
            network_magic: 1,
            nonce: [5u8; 32],
        };
        let bundle = attest_bundle(claim.clone(), &[&k1, &k2], false).unwrap();
        let auth = vec![k1.public_key_uncompressed(), k2.public_key_uncompressed()];
        verify_threshold(&claim, &bundle.signatures, &auth, 2, false).unwrap();
    }
}
