//! Neo N3 settlement attestations for neo-zkvm.
//!
//! Current Neo N3 cannot cheaply verify Groth16 on-chain (no pairing precompile).
//! This crate defines the **canonical message** attestors sign after verifying an
//! SP1 proof off-chain, and helpers to check **N-of-M secp256r1 ECDSA**.

use ecdsa::signature::{Signer, Verifier};
use neo_vm_guest::{ProofMode, PublicInputs};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Domain tag for attestation digests (UTF-8).
pub const ATTESTATION_DOMAIN: &[u8] = b"neo-zkvm-attestation-v1";

/// Wire encoding of [`ProofMode`] for digests and contracts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProofModeCode {
    Execute = 0,
    Mock = 1,
    Sp1 = 2,
    Plonk = 3,
    Groth16 = 4,
}

impl From<ProofMode> for ProofModeCode {
    fn from(mode: ProofMode) -> Self {
        match mode {
            ProofMode::Execute => Self::Execute,
            ProofMode::Mock => Self::Mock,
            ProofMode::Sp1 => Self::Sp1,
            ProofMode::Plonk => Self::Plonk,
            ProofMode::Groth16 => Self::Groth16,
        }
    }
}

impl ProofModeCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Execute),
            1 => Some(Self::Mock),
            2 => Some(Self::Sp1),
            3 => Some(Self::Plonk),
            4 => Some(Self::Groth16),
            _ => None,
        }
    }

    /// Production settlement must not accept Execute/Mock.
    pub fn is_settlement_safe(self) -> bool {
        matches!(self, Self::Sp1 | Self::Plonk | Self::Groth16)
    }
}

/// Public claim fields that are signed and later checked on Neo N3.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationClaim {
    /// Guest / program identity (e.g. SHA-256 of SP1 verifying key bytes).
    pub program_id: [u8; 32],
    /// Proof system mode that was verified off-chain.
    pub proof_mode: ProofModeCode,
    /// Bound public inputs from neo-zkvm.
    pub public_inputs: PublicInputs,
    /// App-specific claim commitment (threshold, allowlist root, expected n, …).
    pub app_claim_hash: [u8; 32],
    /// Neo network magic (domain separation across networks).
    pub network_magic: u32,
    /// Unique per submission; contract must reject replays.
    pub nonce: [u8; 32],
}

/// One ECDSA signature over the claim digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestorSignature {
    /// Uncompressed SEC1 public key bytes (0x04 ‖ X ‖ Y), 65 bytes.
    pub public_key: Vec<u8>,
    /// Compact signature (r ‖ s), 64 bytes.
    pub signature: Vec<u8>,
}

/// Bundle submitted to a Neo N3 contract (plus optional off-chain proof bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationBundle {
    pub claim: AttestationClaim,
    pub signatures: Vec<AttestorSignature>,
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("proof mode is not safe for settlement (reject Mock/Execute)")]
    UnsafeProofMode,
    #[error("threshold not met: got {got}, need {need}")]
    ThresholdNotMet { got: usize, need: usize },
    #[error("duplicate attestor public key")]
    DuplicateAttestor,
    #[error("public key not in the authorized attestor set")]
    UnauthorizedAttestor,
}

/// Compute the 32-byte SHA-256 digest that attestors sign.
///
/// Layout is fixed so Neo contracts can recompute it field-by-field.
#[must_use]
pub fn attestation_digest(claim: &AttestationClaim) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(ATTESTATION_DOMAIN);
    h.update([0u8]);
    h.update(claim.program_id);
    h.update([claim.proof_mode as u8]);
    h.update(claim.public_inputs.script_hash);
    h.update(claim.public_inputs.input_hash);
    h.update(claim.public_inputs.output_hash);
    h.update(claim.public_inputs.gas_consumed.to_le_bytes());
    h.update([claim.public_inputs.execution_success as u8]);
    h.update(claim.app_claim_hash);
    h.update(claim.network_magic.to_le_bytes());
    h.update(claim.nonce);
    h.finalize().into()
}

/// SHA-256 helper for app-level claims (e.g. expected product `n`).
#[must_use]
pub fn app_claim_hash(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// secp256r1 keypair for local tests and attestor operators.
pub struct AttestorKeypair {
    signing: SigningKey,
}

impl AttestorKeypair {
    pub fn generate() -> Self {
        Self {
            signing: SigningKey::random(&mut OsRng),
        }
    }

    pub fn from_bytes(secret: &[u8; 32]) -> Result<Self, AttestationError> {
        let signing = SigningKey::from_bytes(secret.into())
            .map_err(|_| AttestationError::InvalidPublicKey)?;
        Ok(Self { signing })
    }

    /// Uncompressed SEC1 public key (65 bytes, 0x04-prefixed).
    pub fn public_key_uncompressed(&self) -> Vec<u8> {
        let vk = VerifyingKey::from(&self.signing);
        vk.to_encoded_point(false).as_bytes().to_vec()
    }

    pub fn sign_digest(&self, digest: &[u8; 32]) -> Result<AttestorSignature, AttestationError> {
        let sig: Signature = self.signing.sign(digest);
        Ok(AttestorSignature {
            public_key: self.public_key_uncompressed(),
            signature: sig.to_bytes().to_vec(),
        })
    }
}

/// Sign a claim (rejects unsafe modes unless `allow_unsafe_mode`).
pub fn sign_claim(
    keypair: &AttestorKeypair,
    claim: &AttestationClaim,
    allow_unsafe_mode: bool,
) -> Result<AttestorSignature, AttestationError> {
    if !allow_unsafe_mode && !claim.proof_mode.is_settlement_safe() {
        return Err(AttestationError::UnsafeProofMode);
    }
    let digest = attestation_digest(claim);
    keypair.sign_digest(&digest)
}

fn verify_one(digest: &[u8; 32], sig: &AttestorSignature) -> Result<(), AttestationError> {
    let vk = VerifyingKey::from_sec1_bytes(&sig.public_key)
        .map_err(|_| AttestationError::InvalidPublicKey)?;
    let signature =
        Signature::from_slice(&sig.signature).map_err(|_| AttestationError::InvalidSignature)?;
    vk.verify(digest, &signature)
        .map_err(|_| AttestationError::InvalidSignature)
}

/// Verify N-of-M signatures over the claim digest.
///
/// `authorized` is the set of uncompressed public keys allowed to attest.
/// Only signatures from distinct authorized keys count toward `threshold`.
pub fn verify_threshold(
    claim: &AttestationClaim,
    signatures: &[AttestorSignature],
    authorized: &[Vec<u8>],
    threshold: usize,
    allow_unsafe_mode: bool,
) -> Result<(), AttestationError> {
    if !allow_unsafe_mode && !claim.proof_mode.is_settlement_safe() {
        return Err(AttestationError::UnsafeProofMode);
    }
    if threshold == 0 {
        return Err(AttestationError::ThresholdNotMet { got: 0, need: 1 });
    }

    let digest = attestation_digest(claim);
    let mut seen: Vec<Vec<u8>> = Vec::new();
    let mut valid = 0usize;

    for sig in signatures {
        if !authorized.iter().any(|k| k == &sig.public_key) {
            return Err(AttestationError::UnauthorizedAttestor);
        }
        if seen.iter().any(|k| k == &sig.public_key) {
            return Err(AttestationError::DuplicateAttestor);
        }
        verify_one(&digest, sig)?;
        seen.push(sig.public_key.clone());
        valid += 1;
    }

    if valid < threshold {
        return Err(AttestationError::ThresholdNotMet {
            got: valid,
            need: threshold,
        });
    }
    Ok(())
}

/// Convenience: build a bundle from multiple keypairs.
pub fn attest_bundle(
    claim: AttestationClaim,
    keypairs: &[&AttestorKeypair],
    allow_unsafe_mode: bool,
) -> Result<AttestationBundle, AttestationError> {
    let mut signatures = Vec::with_capacity(keypairs.len());
    for kp in keypairs {
        signatures.push(sign_claim(kp, &claim, allow_unsafe_mode)?);
    }
    Ok(AttestationBundle { claim, signatures })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_claim(mode: ProofModeCode) -> AttestationClaim {
        AttestationClaim {
            program_id: [7u8; 32],
            proof_mode: mode,
            public_inputs: PublicInputs {
                script_hash: [1u8; 32],
                input_hash: [2u8; 32],
                output_hash: [3u8; 32],
                gas_consumed: 42,
                execution_success: true,
            },
            app_claim_hash: app_claim_hash(b"n=221"),
            network_magic: 0x334f_454e, // example magic bytes
            nonce: [9u8; 32],
        }
    }

    #[test]
    fn digest_is_stable() {
        let c = sample_claim(ProofModeCode::Groth16);
        let a = attestation_digest(&c);
        let b = attestation_digest(&c);
        assert_eq!(a, b);
        assert_ne!(a, [0u8; 32]);
    }

    #[test]
    fn digest_changes_when_nonce_changes() {
        let mut c = sample_claim(ProofModeCode::Groth16);
        let d1 = attestation_digest(&c);
        c.nonce[0] ^= 1;
        let d2 = attestation_digest(&c);
        assert_ne!(d1, d2);
    }

    #[test]
    fn n_of_m_threshold_passes() {
        let k1 = AttestorKeypair::generate();
        let k2 = AttestorKeypair::generate();
        let k3 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let bundle = attest_bundle(claim.clone(), &[&k1, &k2], false).unwrap();
        let authorized = vec![
            k1.public_key_uncompressed(),
            k2.public_key_uncompressed(),
            k3.public_key_uncompressed(),
        ];
        verify_threshold(&bundle.claim, &bundle.signatures, &authorized, 2, false).unwrap();
    }

    #[test]
    fn threshold_fails_with_one_of_two() {
        let k1 = AttestorKeypair::generate();
        let k2 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Plonk);
        let bundle = attest_bundle(claim.clone(), &[&k1], false).unwrap();
        let authorized = vec![k1.public_key_uncompressed(), k2.public_key_uncompressed()];
        let err = verify_threshold(&claim, &bundle.signatures, &authorized, 2, false).unwrap_err();
        assert!(matches!(
            err,
            AttestationError::ThresholdNotMet { got: 1, need: 2 }
        ));
    }

    #[test]
    fn rejects_mock_mode_for_settlement() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Mock);
        let err = sign_claim(&k1, &claim, false).unwrap_err();
        assert!(matches!(err, AttestationError::UnsafeProofMode));
        // Explicit opt-in for tests/demos
        let sig = sign_claim(&k1, &claim, true).unwrap();
        let authorized = vec![k1.public_key_uncompressed()];
        verify_threshold(&claim, &[sig], &authorized, 1, true).unwrap();
    }

    #[test]
    fn rejects_unauthorized_key() {
        let k1 = AttestorKeypair::generate();
        let k2 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Sp1);
        let sig = sign_claim(&k1, &claim, false).unwrap();
        let authorized = vec![k2.public_key_uncompressed()];
        let err = verify_threshold(&claim, &[sig], &authorized, 1, false).unwrap_err();
        assert!(matches!(err, AttestationError::UnauthorizedAttestor));
    }

    #[test]
    fn proof_mode_from_neo_guest() {
        assert_eq!(
            ProofModeCode::from(ProofMode::Groth16),
            ProofModeCode::Groth16
        );
        assert!(ProofModeCode::Groth16.is_settlement_safe());
        assert!(!ProofModeCode::Mock.is_settlement_safe());
    }
}
