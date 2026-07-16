//! Fuzz Neo N3 Path A attestation: digests, N-of-M ECDSA, config, JSON, tamper resistance.
//!
//! Invariants:
//! - `attestation_digest` is pure and deterministic
//! - Valid N-of-M signatures always verify
//! - Tampered claims / unauthorized keys / duplicates / below-threshold never verify
//! - Mock/Execute rejected unless `allow_unsafe_mode`
//! - Hex/JSON/config parsers never panic

#![no_main]

use libfuzzer_sys::fuzz_target;
use neo_vm_guest::PublicInputs;
use neo_zkvm_attestation::{
    AttestationClaim, AttestationConfig, AttestorKeypair, BundleJson, ClaimJson, ProofModeCode,
    app_claim_hash, attest_bundle, attestation_digest, claim_from_public_inputs, parse_hex32,
    parse_hex_bytes, program_id_from_vkey_hash, sign_claim, verify_bundle, verify_threshold,
};

/// Fixed valid secp256r1 secrets (never all-zero / out-of-range).
const SK1: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32,
    0x10, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
    0xff, 0x01,
];
const SK2: [u8; 32] = [
    0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
    0x99, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0,
    0xf0, 0x02,
];
const SK3: [u8; 32] = [
    0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76,
    0x87, 0x03,
];

fn mode_from_byte(b: u8) -> ProofModeCode {
    match b % 5 {
        0 => ProofModeCode::Execute,
        1 => ProofModeCode::Mock,
        2 => ProofModeCode::Sp1,
        3 => ProofModeCode::Plonk,
        _ => ProofModeCode::Groth16,
    }
}

fn fill32(data: &[u8], offset: usize, xor: u8) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..32 {
        let b = data.get(offset.wrapping_add(i)).copied().unwrap_or(0);
        out[i] = b ^ xor.wrapping_add(i as u8);
    }
    // Keep program_id non-zero so claim_from_public_inputs can succeed.
    if out == [0u8; 32] {
        out[0] = 1;
    }
    out
}

fn claim_from_data(data: &[u8]) -> AttestationClaim {
    let mode = mode_from_byte(data.first().copied().unwrap_or(4));
    AttestationClaim {
        program_id: fill32(data, 1, 0x11),
        proof_mode: mode,
        public_inputs: PublicInputs {
            script_hash: fill32(data, 17, 0x22),
            input_hash: fill32(data, 33, 0x33),
            output_hash: fill32(data, 49, 0x44),
            gas_consumed: u64::from_le_bytes([
                data.get(65).copied().unwrap_or(0),
                data.get(66).copied().unwrap_or(0),
                data.get(67).copied().unwrap_or(0),
                data.get(68).copied().unwrap_or(0),
                data.get(69).copied().unwrap_or(0),
                data.get(70).copied().unwrap_or(0),
                data.get(71).copied().unwrap_or(0),
                data.get(72).copied().unwrap_or(0),
            ]),
            execution_success: data.get(73).copied().unwrap_or(1) & 1 == 1,
        },
        app_claim_hash: app_claim_hash(data.get(74..).unwrap_or(&[])),
        network_magic: u32::from_le_bytes([
            data.get(80).copied().unwrap_or(0x4e),
            data.get(81).copied().unwrap_or(0x45),
            data.get(82).copied().unwrap_or(0x4f),
            data.get(83).copied().unwrap_or(0x33),
        ]),
        nonce: fill32(data, 84, 0x55),
    }
}

fuzz_target!(|data: &[u8]| {
    // --- 1) Parsers must never panic ---
    let _ = parse_hex_bytes(&hex::encode(data));
    let _ = parse_hex_bytes(std::str::from_utf8(data).unwrap_or(""));
    let _ = parse_hex32(&hex::encode(fill32(data, 0, 0)));
    let _ = program_id_from_vkey_hash(&fill32(data, 0, 0x7a));
    let _ = app_claim_hash(data);

    // Adversarial config / claim JSON (no panic).
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<AttestationConfig>(s);
        let _ = serde_json::from_str::<ClaimJson>(s);
        let _ = serde_json::from_str::<BundleJson>(s);
    }

    if data.is_empty() {
        return;
    }

    let k1 = AttestorKeypair::from_bytes(&SK1).expect("SK1 valid");
    let k2 = AttestorKeypair::from_bytes(&SK2).expect("SK2 valid");
    let k3 = AttestorKeypair::from_bytes(&SK3).expect("SK3 valid");
    let authorized = vec![
        k1.public_key_uncompressed(),
        k2.public_key_uncompressed(),
        k3.public_key_uncompressed(),
    ];

    let claim = claim_from_data(data);
    let allow_unsafe = claim.proof_mode == ProofModeCode::Mock
        || claim.proof_mode == ProofModeCode::Execute
        || (data[0] & 0x80) != 0;

    // --- 2) Digest determinism ---
    let d1 = attestation_digest(&claim);
    let d2 = attestation_digest(&claim);
    assert_eq!(d1, d2, "digest must be deterministic");

    // Changing any field must change the digest.
    let mut claim2 = claim.clone();
    claim2.nonce[0] ^= 0x01;
    assert_ne!(
        attestation_digest(&claim),
        attestation_digest(&claim2),
        "nonce flip must change digest"
    );
    claim2 = claim.clone();
    claim2.public_inputs.gas_consumed = claim2.public_inputs.gas_consumed.wrapping_add(1);
    assert_ne!(
        attestation_digest(&claim),
        attestation_digest(&claim2),
        "gas flip must change digest"
    );
    claim2 = claim.clone();
    claim2.network_magic = claim2.network_magic.wrapping_add(1);
    assert_ne!(
        attestation_digest(&claim),
        attestation_digest(&claim2),
        "network_magic flip must change digest"
    );

    // --- 3) claim_from_public_inputs rejects zero program_id ---
    let zero = claim_from_public_inputs(
        [0u8; 32],
        claim.proof_mode,
        claim.public_inputs.clone(),
        claim.app_claim_hash,
        claim.network_magic,
        claim.nonce,
    );
    assert!(zero.is_err());

    // --- 4) Sign + threshold happy path (2-of-3) ---
    let sign_result = attest_bundle(claim.clone(), &[&k1, &k2], allow_unsafe);
    match sign_result {
        Ok(bundle) => {
            verify_threshold(
                &bundle.claim,
                &bundle.signatures,
                &authorized,
                2,
                allow_unsafe,
            )
            .expect("valid 2-of-3 must verify");

            // Below threshold fails.
            let one = attest_bundle(claim.clone(), &[&k1], allow_unsafe).expect("one sig");
            assert!(
                verify_threshold(&one.claim, &one.signatures, &authorized, 2, allow_unsafe)
                    .is_err(),
                "1-of-2 must fail threshold 2"
            );

            // Unauthorized key fails.
            let outsider = AttestorKeypair::generate();
            if let Ok(bad_sig) = sign_claim(&outsider, &claim, allow_unsafe) {
                assert!(
                    verify_threshold(&claim, &[bad_sig], &authorized, 1, allow_unsafe).is_err(),
                    "outsider must be rejected"
                );
            }

            // Duplicate signatures rejected.
            if let Ok(sig) = sign_claim(&k1, &claim, allow_unsafe) {
                assert!(
                    verify_threshold(
                        &claim,
                        &[sig.clone(), sig],
                        &authorized,
                        1,
                        allow_unsafe
                    )
                    .is_err(),
                    "duplicate attestor must be rejected"
                );
            }

            // Tampered claim with original signatures must fail.
            let mut tampered = bundle.claim.clone();
            tampered.app_claim_hash[0] ^= 0xFF;
            assert!(
                verify_threshold(
                    &tampered,
                    &bundle.signatures,
                    &authorized,
                    2,
                    allow_unsafe
                )
                .is_err(),
                "tampered claim must not verify"
            );

            // JSON round-trip preserves verification.
            let json = BundleJson::from(&bundle);
            if let Ok(restored) = json.into_bundle() {
                verify_threshold(
                    &restored.claim,
                    &restored.signatures,
                    &authorized,
                    2,
                    allow_unsafe,
                )
                .expect("JSON round-trip must still verify");
            }

            // Config-based verify_bundle.
            let cfg = AttestationConfig {
                program_id: hex::encode(bundle.claim.program_id),
                network_magic: bundle.claim.network_magic,
                threshold: 2,
                attestors: authorized.iter().map(hex::encode).collect(),
            };
            if let Ok(parsed) = cfg.parse() {
                verify_bundle(&bundle, &parsed, allow_unsafe)
                    .expect("config verify must accept valid bundle");

                // Wrong magic fails.
                let mut bad_cfg = parsed.clone();
                bad_cfg.network_magic = parsed.network_magic.wrapping_add(1);
                assert!(verify_bundle(&bundle, &bad_cfg, allow_unsafe).is_err());
            }
        }
        Err(_) => {
            // Unsafe mode rejected without allow flag — expected for Mock/Execute.
            if !allow_unsafe
                && matches!(
                    claim.proof_mode,
                    ProofModeCode::Mock | ProofModeCode::Execute
                )
            {
                // Correct rejection.
            } else if !claim.proof_mode.is_settlement_safe() && !allow_unsafe {
                // OK
            }
            // Settlement-safe modes must always sign successfully with valid keys.
            if claim.proof_mode.is_settlement_safe() {
                panic!("settlement-safe claim must sign: mode={:?}", claim.proof_mode);
            }
        }
    }

    // --- 5) Mock/Execute always blocked when allow_unsafe=false ---
    if matches!(
        claim.proof_mode,
        ProofModeCode::Mock | ProofModeCode::Execute
    ) {
        assert!(
            sign_claim(&k1, &claim, false).is_err(),
            "Mock/Execute must reject without allow_unsafe_mode"
        );
    }

    // --- 6) Adversarial signature blobs never panic verify ---
    if data.len() >= 64 {
        use neo_zkvm_attestation::AttestorSignature;
        let garbage = AttestorSignature {
            public_key: k1.public_key_uncompressed(),
            signature: data[..64.min(data.len())].to_vec(),
        };
        let _ = verify_threshold(&claim, &[garbage], &authorized, 1, true);
    }
});
