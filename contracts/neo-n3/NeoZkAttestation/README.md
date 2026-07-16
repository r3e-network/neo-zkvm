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

**Does not** verify Groth16 on-chain. Neo BLS12-381 (when present) is **not** used here — stock SP1 EVM Groth16 is BN254, and pairing precompiles are not assumed. Trust = SP1 soundness off-chain + ECDSA threshold on-chain.

## Hash / signature convention

| Side | Behavior |
| --- | --- |
| Rust `attestation_digest` | SHA-256(preimage) → 32-byte digest |
| Rust `p256` `Signer::sign(digest)` | ECDSA over SHA-256(digest) |
| Neo `VerifyWithECDsa(digest, pk, sig, secp256r1SHA256)` | ECDSA over SHA-256(digest) |

Both sides double-hash the preimage. Do not change one without the other.

## Deploy parameters

| Storage | Meaning |
| --- | --- |
| `ProgramId` | 32-byte guest/program commitment (non-zero) |
| `Attestors` | Authorized SEC1 pubkeys (33 or 65 bytes) |
| `Threshold` | N-of-M |
| `UsedNonces` | Map nonce → used |
| `NetworkMagic` | Must match claim |

`Initialize` may be called **once**.

## Operator tooling

```bash
# Keygen + config (see docs/cli.md)
neo-zkvm attest keygen -o attestor1.json
neo-zkvm prove script.bin -m mock -o proof.bin
neo-zkvm attest bundle --proof proof.bin -m mock --config committee.json \
  --secret-key <hex> --secret-key <hex> --allow-unsafe-mode -o bundle.json
neo-zkvm attest check --bundle bundle.json --config committee.json --allow-unsafe-mode
```

Production: use `-m groth16` / `sp1` / `plonk` and **omit** `--allow-unsafe-mode`.

## Production rules

- Reject `proof_mode` ∈ {0 Execute, 1 Mock}.
- Accept only 2 Sp1 / 3 Plonk / 4 Groth16.
- App layer additionally checks `app_claim_hash` / public inputs for business rules.
- Anyone can re-verify the original SP1 proof off-chain against public data.

See `NeoZkAttestation.cs` (Neo N3 DevPack / neo-express).
)