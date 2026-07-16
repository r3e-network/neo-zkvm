# NeoZkAttestation (Neo N3 contract)

Settlement contract for **current Neo N3** (deploy only; no protocol changes).

## Model (Path A — works today)

Off-chain attestors verify an SP1 (Groth16/PLONK) neo-zkvm proof, then sign:

```text
SHA256( neo-zkvm-attestation-v1 || 0x00 || program_id || … || nonce )
```

This contract:

1. Recomputes the digest from submitted fields (little-endian multi-byte ints).
2. Counts valid `VerifyWithECDsa` (secp256r1 + SHA256) signatures from the authorized set.
3. Rejects **duplicate** attestors in a single submission.
4. Requires `count >= Threshold`.
5. Rejects replayed `nonce` values and Mock/Execute proof modes.
6. Emits `ClaimSettled` and stores the last app claim + digest.

**Storage key note:** attestors are stored as `0x02 || SHA256(pubkey)` because Neo
storage keys are limited to 64 bytes (uncompressed SEC1 is 65). Prefer
**compressed 33-byte** pubkeys for `Initialize` / `Submit`.

## Hash / signature convention

| Side | Behavior |
| --- | --- |
| Rust `attestation_digest` | SHA-256(preimage) → 32-byte digest |
| Rust `p256` `Signer::sign(digest)` | ECDSA over SHA-256(digest) |
| Neo `VerifyWithECDsa(digest, pk, sig, secp256r1SHA256)` | ECDSA over SHA-256(digest) |

## Build

```bash
# requires nccs (Neo.Compiler.CSharp) + .NET 10
nccs NeoZkAttestation.csproj
# outputs bin/sc/NeoZkAttestation.nef + .manifest.json
```

## Testnet validation (T5)

Validated on Neo N3 T5 (`magic=894710606`, RPC `http://seed1t5.neo.org:20332`):

| Step | Result |
| --- | --- |
| Off-chain mock prove + 2-of-3 ECDSA | OK |
| Deploy contract | OK |
| Initialize (threshold=2) | OK |
| Submit Mock mode | rejected (`false`) |
| Submit Sp1 mode (demo claim) | OK + `ClaimSettled` |
| Nonce replay | rejected (`false`) |

Example live deployment (T5):

- Contract: `0x8fcf08538ad4d750cfe3f48d263079fd32e59acb`
- Submit tx: `0x35f7c336740432900359f8e7e347976c00b6e965ac5fe87cd9a804830c93d8d4`

Production: use real SP1/Groth16 proofs and omit Mock mode entirely.
