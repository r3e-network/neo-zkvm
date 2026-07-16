# NeoZkAttestation (Neo N3 contract sketch)

Settlement contract for **current Neo N3** (deploy only; no protocol changes).

## Model

Off-chain attestors verify an SP1 (Groth16/PLONK) neo-zkvm proof, then sign:

```text
SHA256( neo-zkvm-attestation-v1 || 0x00 || program_id || … || nonce )
```

This contract:

1. Recomputes the digest from submitted fields.
2. Counts valid `VerifyWithECDsa` (secp256r1) signatures from the authorized set.
3. Requires `count >= Threshold`.
4. Rejects replayed `nonce` values.
5. Emits an event / updates storage for the app claim.

**Does not** verify Groth16 on-chain (Neo has no pairing precompile). Trust = crypto of SP1 off-chain + ECDSA threshold on-chain.

## Deploy parameters

| Storage | Meaning |
| --- | --- |
| `ProgramId` | 32-byte guest/program commitment |
| `Attestors` | List of authorized compressed/uncompressed pubkeys |
| `Threshold` | N-of-M |
| `UsedNonces` | Map nonce → bool |
| `NetworkMagic` | Must match claim |

## Production rules

- Reject `proof_mode` ∈ {0 Execute, 1 Mock}.
- Accept only 2 Sp1 / 3 Plonk / 4 Groth16.
- App layer additionally checks `app_claim_hash` / public inputs for business rules.

See `NeoZkAttestation.cs` for the sketch (compile with Neo N3 DevPack / neo-express).
