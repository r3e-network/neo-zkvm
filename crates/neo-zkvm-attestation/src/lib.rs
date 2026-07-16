//! Neo N3 settlement attestations for neo-zkvm.
//!
//! Current Neo N3 cannot cheaply verify Groth16 on-chain (no pairing precompile).
//! This crate defines the **canonical message** attestors sign after verifying an
//! SP1 proof off-chain, and helpers to check **N-of-M secp256r1 ECDSA**.
//!
//! Production settlement path (Path A): off-chain SP1 verify → attestors sign →
//! Neo contract `VerifyWithECDsa` threshold. BLS12-381 alone does not replace this
//! unless Neo exposes matching pairings *and* the proof curve matches (see docs).

use ecdsa::signature::{Signer, Verifier};
use neo_vm_guest::{ProofMode, PublicInputs};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use rand_core::{OsRng, RngCore};
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

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Execute => "execute",
            Self::Mock => "mock",
            Self::Sp1 => "sp1",
            Self::Plonk => "plonk",
            Self::Groth16 => "groth16",
        }
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

/// Operator config for a settlement committee (JSON-friendly).
///
/// Public keys are uncompressed SEC1 hex (`04` ‖ X ‖ Y).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationConfig {
    /// 32-byte program id as hex (64 hex chars, optional 0x prefix).
    pub program_id: String,
    /// Neo network magic (mainnet / testnet / private).
    pub network_magic: u32,
    /// Required number of distinct valid attestor signatures.
    pub threshold: usize,
    /// Authorized attestor public keys (uncompressed SEC1 hex).
    pub attestors: Vec<String>,
}

impl AttestationConfig {
    /// Parse and validate config fields into binary form.
    pub fn parse(&self) -> Result<ParsedAttestationConfig, AttestationError> {
        let program_id = parse_hex32(&self.program_id)?;
        if self.threshold == 0 {
            return Err(AttestationError::ThresholdNotMet { got: 0, need: 1 });
        }
        if self.attestors.is_empty() {
            return Err(AttestationError::EmptyAttestorSet);
        }
        if self.threshold > self.attestors.len() {
            return Err(AttestationError::ThresholdNotMet {
                got: self.attestors.len(),
                need: self.threshold,
            });
        }
        let mut attestors = Vec::with_capacity(self.attestors.len());
        for hex_key in &self.attestors {
            let pk = parse_hex_bytes(hex_key)?;
            if pk.len() != 65 || pk[0] != 0x04 {
                return Err(AttestationError::InvalidPublicKey);
            }
            // Reject duplicates in the authorized set.
            if attestors.iter().any(|k: &Vec<u8>| k == &pk) {
                return Err(AttestationError::DuplicateAttestor);
            }
            // Validate SEC1 parse.
            VerifyingKey::from_sec1_bytes(&pk).map_err(|_| AttestationError::InvalidPublicKey)?;
            attestors.push(pk);
        }
        Ok(ParsedAttestationConfig {
            program_id,
            network_magic: self.network_magic,
            threshold: self.threshold,
            attestors,
        })
    }
}

/// Binary form of [`AttestationConfig`] after hex parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedAttestationConfig {
    pub program_id: [u8; 32],
    pub network_magic: u32,
    pub threshold: usize,
    pub attestors: Vec<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid secret key")]
    InvalidSecretKey,
    #[error("proof mode is not safe for settlement (reject Mock/Execute)")]
    UnsafeProofMode,
    #[error("threshold not met: got {got}, need {need}")]
    ThresholdNotMet { got: usize, need: usize },
    #[error("duplicate attestor public key")]
    DuplicateAttestor,
    #[error("public key not in the authorized attestor set")]
    UnauthorizedAttestor,
    #[error("empty attestor set")]
    EmptyAttestorSet,
    #[error("invalid hex: {0}")]
    InvalidHex(String),
    #[error("expected {expected}-byte value, got {got}")]
    InvalidLength { expected: usize, got: usize },
    #[error("program_id is all zeros; set an explicit program identity")]
    ZeroProgramId,
    #[error("program_id does not match operator config")]
    ProgramIdMismatch,
    #[error("network_magic does not match operator config")]
    NetworkMagicMismatch,
}

/// Compute the 32-byte SHA-256 digest that attestors sign.
///
/// Layout is fixed so Neo contracts can recompute it field-by-field.
///
/// **Signing note:** Rust `sign_digest` and Neo `VerifyWithECDsa(..., secp256r1SHA256)`
/// both apply SHA-256 to this 32-byte value before ECDSA (double-hash of the
/// preimage). Keep both sides on the same convention.
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

/// Build a claim from public inputs (typically taken from a verified [`neo_vm_guest::NeoProof`]).
pub fn claim_from_public_inputs(
    program_id: [u8; 32],
    proof_mode: ProofModeCode,
    public_inputs: PublicInputs,
    app_claim_hash: [u8; 32],
    network_magic: u32,
    nonce: [u8; 32],
) -> Result<AttestationClaim, AttestationError> {
    if program_id == [0u8; 32] {
        return Err(AttestationError::ZeroProgramId);
    }
    Ok(AttestationClaim {
        program_id,
        proof_mode,
        public_inputs,
        app_claim_hash,
        network_magic,
        nonce,
    })
}

/// Prefer a non-zero SP1 `vkey_hash` as `program_id`; otherwise return `None`.
#[must_use]
pub fn program_id_from_vkey_hash(vkey_hash: &[u8; 32]) -> Option<[u8; 32]> {
    if *vkey_hash == [0u8; 32] {
        None
    } else {
        Some(*vkey_hash)
    }
}

/// Cryptographically random 32-byte nonce for replay protection.
#[must_use]
pub fn random_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Parse 32-byte hex (optional `0x` prefix).
pub fn parse_hex32(s: &str) -> Result<[u8; 32], AttestationError> {
    let bytes = parse_hex_bytes(s)?;
    if bytes.len() != 32 {
        return Err(AttestationError::InvalidLength {
            expected: 32,
            got: bytes.len(),
        });
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Parse hex bytes (optional `0x` prefix).
pub fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, AttestationError> {
    let hex_str = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    hex::decode(hex_str).map_err(|e| AttestationError::InvalidHex(e.to_string()))
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
        let signing =
            SigningKey::from_bytes(secret.into()).map_err(|_| AttestationError::InvalidSecretKey)?;
        Ok(Self { signing })
    }

    pub fn from_hex(secret_hex: &str) -> Result<Self, AttestationError> {
        let bytes = parse_hex32(secret_hex)?;
        Self::from_bytes(&bytes)
    }

    /// 32-byte secret key (handle as sensitive material).
    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing.to_bytes().into()
    }

    /// Uncompressed SEC1 public key (65 bytes, 0x04-prefixed).
    pub fn public_key_uncompressed(&self) -> Vec<u8> {
        let vk = VerifyingKey::from(&self.signing);
        vk.to_encoded_point(false).as_bytes().to_vec()
    }

    /// Sign the 32-byte attestation digest.
    ///
    /// Uses ECDSA-SHA256 over the digest bytes (same as Neo
    /// `VerifyWithECDsa(digest, pk, sig, secp256r1SHA256)`).
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
    // SEC1: compressed 33 bytes (02/03‖X) or uncompressed 65 bytes (04‖X‖Y).
    let pk_len = sig.public_key.len();
    if pk_len != 33 && pk_len != 65 {
        return Err(AttestationError::InvalidPublicKey);
    }
    // Compact (r ‖ s) only — 64 bytes. Reject DER/mixed lengths early.
    if sig.signature.len() != 64 {
        return Err(AttestationError::InvalidSignature);
    }
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
    if authorized.is_empty() {
        return Err(AttestationError::EmptyAttestorSet);
    }
    if threshold == 0 {
        return Err(AttestationError::ThresholdNotMet { got: 0, need: 1 });
    }
    if threshold > authorized.len() {
        return Err(AttestationError::ThresholdNotMet {
            got: 0,
            need: threshold,
        });
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

/// Verify a bundle against a parsed operator config.
pub fn verify_bundle(
    bundle: &AttestationBundle,
    config: &ParsedAttestationConfig,
    allow_unsafe_mode: bool,
) -> Result<(), AttestationError> {
    if bundle.claim.program_id != config.program_id {
        return Err(AttestationError::ProgramIdMismatch);
    }
    if bundle.claim.network_magic != config.network_magic {
        return Err(AttestationError::NetworkMagicMismatch);
    }
    verify_threshold(
        &bundle.claim,
        &bundle.signatures,
        &config.attestors,
        config.threshold,
        allow_unsafe_mode,
    )
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

/// Wire-friendly claim with hex fields (for CLI JSON).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimJson {
    pub program_id: String,
    pub proof_mode: String,
    pub script_hash: String,
    pub input_hash: String,
    pub output_hash: String,
    pub gas_consumed: u64,
    pub execution_success: bool,
    pub app_claim_hash: String,
    pub network_magic: u32,
    pub nonce: String,
}

impl From<&AttestationClaim> for ClaimJson {
    fn from(c: &AttestationClaim) -> Self {
        Self {
            program_id: hex::encode(c.program_id),
            proof_mode: c.proof_mode.as_str().to_string(),
            script_hash: hex::encode(c.public_inputs.script_hash),
            input_hash: hex::encode(c.public_inputs.input_hash),
            output_hash: hex::encode(c.public_inputs.output_hash),
            gas_consumed: c.public_inputs.gas_consumed,
            execution_success: c.public_inputs.execution_success,
            app_claim_hash: hex::encode(c.app_claim_hash),
            network_magic: c.network_magic,
            nonce: hex::encode(c.nonce),
        }
    }
}

impl ClaimJson {
    pub fn into_claim(self) -> Result<AttestationClaim, AttestationError> {
        let proof_mode = match self.proof_mode.to_ascii_lowercase().as_str() {
            "execute" => ProofModeCode::Execute,
            "mock" => ProofModeCode::Mock,
            "sp1" => ProofModeCode::Sp1,
            "plonk" => ProofModeCode::Plonk,
            "groth16" => ProofModeCode::Groth16,
            other => {
                return Err(AttestationError::InvalidHex(format!(
                    "unknown proof_mode '{other}'"
                )));
            }
        };
        Ok(AttestationClaim {
            program_id: parse_hex32(&self.program_id)?,
            proof_mode,
            public_inputs: PublicInputs {
                script_hash: parse_hex32(&self.script_hash)?,
                input_hash: parse_hex32(&self.input_hash)?,
                output_hash: parse_hex32(&self.output_hash)?,
                gas_consumed: self.gas_consumed,
                execution_success: self.execution_success,
            },
            app_claim_hash: parse_hex32(&self.app_claim_hash)?,
            network_magic: self.network_magic,
            nonce: parse_hex32(&self.nonce)?,
        })
    }
}

/// Wire-friendly signature (hex).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureJson {
    pub public_key: String,
    pub signature: String,
}

impl From<&AttestorSignature> for SignatureJson {
    fn from(s: &AttestorSignature) -> Self {
        Self {
            public_key: hex::encode(&s.public_key),
            signature: hex::encode(&s.signature),
        }
    }
}

impl SignatureJson {
    pub fn into_signature(self) -> Result<AttestorSignature, AttestationError> {
        Ok(AttestorSignature {
            public_key: parse_hex_bytes(&self.public_key)?,
            signature: parse_hex_bytes(&self.signature)?,
        })
    }
}

/// Wire-friendly bundle for CLI / attestor service interchange.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleJson {
    pub claim: ClaimJson,
    pub signatures: Vec<SignatureJson>,
    /// Hex of the 32-byte attestation digest (informational; recompute on verify).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

impl From<&AttestationBundle> for BundleJson {
    fn from(b: &AttestationBundle) -> Self {
        let digest = attestation_digest(&b.claim);
        Self {
            claim: ClaimJson::from(&b.claim),
            signatures: b.signatures.iter().map(SignatureJson::from).collect(),
            digest: Some(hex::encode(digest)),
        }
    }
}

impl BundleJson {
    pub fn into_bundle(self) -> Result<AttestationBundle, AttestationError> {
        let claim = self.claim.into_claim()?;
        let mut signatures = Vec::with_capacity(self.signatures.len());
        for s in self.signatures {
            signatures.push(s.into_signature()?);
        }
        Ok(AttestationBundle { claim, signatures })
    }
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

    #[test]
    fn json_roundtrip_bundle() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let bundle = attest_bundle(claim, &[&k1], false).unwrap();
        let json = BundleJson::from(&bundle);
        let restored = json.into_bundle().unwrap();
        assert_eq!(bundle.claim, restored.claim);
        assert_eq!(bundle.signatures, restored.signatures);
    }

    #[test]
    fn config_parse_and_verify_bundle() {
        let k1 = AttestorKeypair::generate();
        let k2 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let bundle = attest_bundle(claim, &[&k1, &k2], false).unwrap();
        let cfg = AttestationConfig {
            program_id: hex::encode(bundle.claim.program_id),
            network_magic: bundle.claim.network_magic,
            threshold: 2,
            attestors: vec![
                hex::encode(k1.public_key_uncompressed()),
                hex::encode(k2.public_key_uncompressed()),
            ],
        };
        let parsed = cfg.parse().unwrap();
        verify_bundle(&bundle, &parsed, false).unwrap();
    }

    #[test]
    fn keypair_from_hex_roundtrip() {
        let k = AttestorKeypair::generate();
        let hex_sk = hex::encode(k.to_bytes());
        let k2 = AttestorKeypair::from_hex(&hex_sk).unwrap();
        assert_eq!(k.public_key_uncompressed(), k2.public_key_uncompressed());
    }

    #[test]
    fn rejects_zero_program_id() {
        let err = claim_from_public_inputs(
            [0u8; 32],
            ProofModeCode::Groth16,
            PublicInputs {
                script_hash: [1u8; 32],
                input_hash: [2u8; 32],
                output_hash: [3u8; 32],
                gas_consumed: 1,
                execution_success: true,
            },
            [4u8; 32],
            1,
            [5u8; 32],
        )
        .unwrap_err();
        assert!(matches!(err, AttestationError::ZeroProgramId));
    }

    #[test]
    fn rejects_duplicate_signatures() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let sig = sign_claim(&k1, &claim, false).unwrap();
        let authorized = vec![k1.public_key_uncompressed()];
        let err =
            verify_threshold(&claim, &[sig.clone(), sig], &authorized, 1, false).unwrap_err();
        assert!(matches!(err, AttestationError::DuplicateAttestor));
    }

    #[test]
    fn rejects_empty_signature_list() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let authorized = vec![k1.public_key_uncompressed()];
        let err = verify_threshold(&claim, &[], &authorized, 1, false).unwrap_err();
        assert!(matches!(
            err,
            AttestationError::ThresholdNotMet { got: 0, need: 1 }
        ));
    }

    #[test]
    fn rejects_empty_authorized_set() {
        let claim = sample_claim(ProofModeCode::Sp1);
        let err = verify_threshold(&claim, &[], &[], 1, false).unwrap_err();
        assert!(matches!(err, AttestationError::EmptyAttestorSet));
    }

    #[test]
    fn rejects_threshold_above_committee_size() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Plonk);
        let sig = sign_claim(&k1, &claim, false).unwrap();
        let authorized = vec![k1.public_key_uncompressed()];
        let err = verify_threshold(&claim, &[sig], &authorized, 2, false).unwrap_err();
        assert!(matches!(
            err,
            AttestationError::ThresholdNotMet { got: 0, need: 2 }
        ));
    }

    #[test]
    fn rejects_wrong_signature_length() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let mut sig = sign_claim(&k1, &claim, false).unwrap();
        sig.signature.pop(); // 63 bytes
        let authorized = vec![k1.public_key_uncompressed()];
        let err = verify_threshold(&claim, &[sig], &authorized, 1, false).unwrap_err();
        assert!(matches!(err, AttestationError::InvalidSignature));
    }

    #[test]
    fn accepts_0x_prefixed_hex_and_config() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let bundle = attest_bundle(claim, &[&k1], false).unwrap();
        let cfg = AttestationConfig {
            program_id: format!("0x{}", hex::encode(bundle.claim.program_id)),
            network_magic: bundle.claim.network_magic,
            threshold: 1,
            attestors: vec![format!("0x{}", hex::encode(k1.public_key_uncompressed()))],
        };
        let parsed = cfg.parse().unwrap();
        verify_bundle(&bundle, &parsed, false).unwrap();
    }

    #[test]
    fn verify_bundle_rejects_program_id_mismatch() {
        let k1 = AttestorKeypair::generate();
        let claim = sample_claim(ProofModeCode::Groth16);
        let bundle = attest_bundle(claim, &[&k1], false).unwrap();
        let mut parsed = AttestationConfig {
            program_id: hex::encode(bundle.claim.program_id),
            network_magic: bundle.claim.network_magic,
            threshold: 1,
            attestors: vec![hex::encode(k1.public_key_uncompressed())],
        }
        .parse()
        .unwrap();
        parsed.program_id[0] ^= 0xFF;
        assert!(matches!(
            verify_bundle(&bundle, &parsed, false),
            Err(AttestationError::ProgramIdMismatch)
        ));
    }

    #[test]
    fn parse_hex_rejects_odd_length() {
        assert!(parse_hex_bytes("abc").is_err());
        assert!(parse_hex32("aa").is_err());
    }
}
