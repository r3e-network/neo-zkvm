# neo-zkvm-attestation

Canonical **attestation digests** and **N-of-M ECDSA** helpers for settling neo-zkvm public claims on **current Neo N3** without protocol changes.

See [docs/neo-n3-attestation.md](../../docs/neo-n3-attestation.md).

## Flow

1. Off-chain: prove NeoVM with SP1 and verify the proof.
2. Attestors sign `attestation_digest(...)`.
3. Neo contract verifies threshold ECDSA and applies business logic.

## Example

```rust
use neo_zkvm_attestation::{AttestationClaim, AttestorKeypair, ProofModeCode, sign_claim, verify_threshold};
```
