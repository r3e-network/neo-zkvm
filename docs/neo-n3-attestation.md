# Neo N3 settlement via attestation (contracts only)

## Why attestation?

Current Neo N3 smart contracts can use **SHA256** and **ECDSA** (`VerifyWithECDsa`), but **not** cheap BN254 pairings like Ethereum’s Groth16 precompile.

Therefore the **mature, deployable** settlement path is:

1. **Off-chain:** prove NeoVM with SP1 (Groth16/PLONK) and verify the proof.
2. **On-chain:** Neo contract checks **N-of-M ECDSA signatures** over a canonical digest of the public claim, then applies business logic.

No Neo protocol changes — only contract deploy.

## Roles

| Role | Responsibility |
| --- | --- |
| **Prover** | Runs `neo-zkvm` / SP1; produces `NeoProof` |
| **Attestor(s)** | Re-verify SP1 proof; sign attestation digest |
| **Neo contract** | Verify threshold signatures + public claim; settle |
| **User / watcher** | Anyone can re-check SP1 off-chain against public data |

## Canonical message

Domain tag (UTF-8, no trailing null):

```text
neo-zkvm-attestation-v1
```

**Signing digest** (32 bytes):

```text
SHA256(
  domain_tag || 0x00 ||
  program_id (32) ||
  proof_mode (u8) ||
  script_hash (32) ||
  input_hash (32) ||
  output_hash (32) ||
  gas_consumed (u64 LE) ||
  execution_success (u8 0/1) ||
  app_claim_hash (32) ||
  network_magic (u32 LE) ||
  nonce (32)
)
```

| Field | Meaning |
| --- | --- |
| `program_id` | Commitment to guest/program identity (e.g. hash of SP1 vkey or ELF id) |
| `proof_mode` | Same encoding as `ProofMode` (see crate): 0=Execute, 1=Mock, 2=Sp1, 3=Plonk, 4=Groth16 |
| `PublicInputs` fields | From `neo_vm_guest::PublicInputs` |
| `app_claim_hash` | App-specific public claim (e.g. SHA256 of expected `n`, root, …) |
| `network_magic` | Neo network magic (mainnet/testnet) for domain separation |
| `nonce` | Unique per submission (replay protection) |

**Production:** contracts **must reject** `proof_mode` ∈ {Execute, Mock}.

## Signature scheme

- Curve: **secp256r1** (NIST P-256) — aligns with common Neo `VerifyWithECDsa` usage.
- Hash: digest above (already SHA-256).
- Encoding: 64-byte compact `(r ‖ s)` for transport; contract uses Neo’s expected format.

## Bundle submitted on-chain

```text
AttestationBundle {
  program_id, proof_mode, public_inputs, app_claim_hash,
  network_magic, nonce,
  signatures: [ { public_key, signature }, ... ]
}
```

Contract:

1. Recompute digest from fields (same layout as Rust).
2. For each signature, `VerifyWithECDsa(digest, pubkey, sig)` if pubkey is in the attestor set.
3. Require `unique_valid >= threshold`.
4. Mark `nonce` used; apply app logic.

## Trust model

| | |
| --- | --- |
| **Crypto trust** | SP1 soundness (off-chain) + ECDSA (on-chain) |
| **Social/ops trust** | Attestor set honesty ≥ threshold |
| **Mitigation** | Public proofs + inputs; anyone can detect false attestation |

This is the standard pattern on chains without SNARK precompiles.

## Rust crate

`neo-zkvm-attestation` — build digest, sign, verify N-of-M.

## Neo contract sketch

See `contracts/neo-n3/NeoZkAttestation/` (C# Neo N3 style sketch).
