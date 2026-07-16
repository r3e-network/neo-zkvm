# neo-zkvm-attestation

Canonical **attestation digests** and **N-of-M ECDSA** helpers for settling neo-zkvm public claims on **current Neo N3** without protocol changes.

See [docs/neo-n3-attestation.md](../../docs/neo-n3-attestation.md) and
[docs/neo-n3-crypto-surface.md](../../docs/neo-n3-crypto-surface.md).

## Flow

1. Off-chain: prove NeoVM with SP1 and verify the proof.
2. Attestors sign `attestation_digest(...)` (secp256r1 ECDSA today).
3. Neo contract verifies threshold ECDSA and applies business logic.

**Note on BLS12-381:** Neo may expose BLS *signatures* and/or *pairings* depending
on node version. That does **not** mean default SP1 Groth16 (BN254) verifies
on-chain. BLS-sig attestors (Path A′) and pairing-based SNARK verify (Path B/C)
are roadmap items gated on CryptoLib APIs.

## Example

```rust
use neo_zkvm_attestation::{AttestationClaim, AttestorKeypair, ProofModeCode, sign_claim, verify_threshold};
```
