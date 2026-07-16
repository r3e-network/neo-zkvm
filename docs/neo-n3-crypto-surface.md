# Neo N3 crypto surface for zkVM settlement

This document separates **what Neo N3 CryptoLib can do**, **what SP1 proofs need**, and **which settlement path is trustworthy**. It is the companion to [neo-n3-attestation.md](neo-n3-attestation.md) and [ROADMAP.md](ROADMAP.md).

## 1. Three different “BLS / pairing” ideas

| Term | Meaning | Enough to verify SP1 Groth16 on-chain? |
| --- | --- | --- |
| **BLS signatures** (Boneh–Lynn–Shacham) | Sign/verify/aggregate on **BLS12-381** groups | **No** — committee signatures, not a SNARK verifier |
| **BLS12-381 pairings** | Multi-pairing check \(e(\cdot,\cdot)\cdots=1\) | **Only if** the proof system uses **BLS12-381** |
| **BN254 pairings** | Same idea on BN254 (alt_bn128) | What **Ethereum + default SP1 EVM wrappers** use |

Having “BLS on Neo” is **not** automatically “we can verify SP1 Groth16 on Neo.”

---

## 2. What SP1 uses for “cheap” verification

Succinct’s production path for EVM-style verify is typically:

```text
SP1 execution  →  wrap  →  Groth16 or PLONK on BN254  →  verify with pairings
```

- Default **EVM verifier** path: **BN254**, not BLS12-381.
- SP1 also has **bn254 / bls12-381 precompiles inside the zkVM guest** (for proving other proofs *inside* SP1) — that is **off-chain proving**, not Neo on-chain verify.

So:

| Goal | Curve / API needed on Neo |
| --- | --- |
| Verify **default SP1 Groth16** on-chain | **BN254 multi-pairing** (not BLS12-381 alone) |
| Verify a **BLS12-381 Groth16/PLONK** on-chain | **BLS12-381 multi-pairing** + a **BLS12-381** proof pipeline |
| Aggregate **attestor signatures** on-chain | **BLS signature verify** (or ECDSA N-of-M) |

---

## 3. Neo N3 CryptoLib inventory (as of neo-project docs / master)

Public Neo.SmartContract.Framework `CryptoLib` surface commonly documented and present on **neo-project/neo** master includes:

| Method | Role for zkVM settlement |
| --- | --- |
| `Sha256` | Digests, commitments, attestation message hash |
| `Ripemd160` / hash helpers (version-dependent) | Neo-style digests |
| `Murmur32` | Non-crypto hashing (not for ZK security) |
| `VerifyWithECDsa` (+ NamedCurveHash) | **secp256r1 / secp256k1** attestor signatures |
| `VerifyWithEd25519` | Alternate attestor scheme |

**BLS12-381 / pairing APIs** are **not** a stable, universally documented part of every Neo N3 release the same way ECDSA is. Core discussions (e.g. v3.9 CryptoLib design, Ethereum-compatible BLS12-381 *aliases*) indicate **interest and possible future expansion**, not “drop-in Groth16 verify today on all Neo N3 deployments.”

**Operator rule:** Pin your **exact Neo node version** and read that build’s `CryptoLib` natives. Do not assume mainnet equals a branch PR.

### How to classify *your* Neo build

```text
If CryptoLib has only ECDSA/Ed25519/SHA256:
  → Settlement path A (attestation) only for trustless-enough deploy

If CryptoLib has BLS Verify / AggregateVerify (signatures):
  → Path A' (attestation with BLS multi-sig) — better ops, still attestors

If CryptoLib has Pairing / multi-pairing check on BN254:
  → Path B possible for default SP1 Groth16

If CryptoLib has Pairing on BLS12-381 only:
  → Path C possible only with a BLS12-381 SNARK wrapper (not stock SP1 EVM path)
```

---

## 4. Settlement paths (no Neo protocol change beyond what natives already ship)

### Path A — ECDSA N-of-M attestation (**default, implemented**)

```text
SP1 verify off-chain → attestors sign SHA256(attestation message) with secp256r1
→ Neo contract: VerifyWithECDsa × N, threshold, nonce, public claim
```

- **Works on current Neo N3** with documented CryptoLib.
- **Crate:** `neo-zkvm-attestation`
- **Contract sketch:** `contracts/neo-n3/NeoZkAttestation`
- **Trust:** SP1 soundness + attestor threshold honesty; public re-verify possible off-chain.

### Path A′ — BLS aggregate attestors (**if** Neo exposes BLS-sig natives)

Same as A, but attestors use **BLS12-381 signatures** and the contract verifies **one aggregate** (or fewer pairings internally inside the native).

- Still **not** full SNARK verify.
- Better gas/ops than many ECDSA checks **if** the native is efficient.
- **Roadmap:** implement when CryptoLib API is confirmed on target Neo version.

### Path B — On-chain SNARK verify on **BN254** (**target — Neo will add BN254**)

```text
SP1 Groth16 (BN254) → Neo CryptoLib BN254 pairings → contract verifies proof + public inputs
```

- Matches **default SP1 EVM** curve family (Succinct / Ethereum Groth16 path).
- **Most trustless** settle: **no ECDSA council**.
- Requires Neo natives (minimum): G1 add/mul, **multi-pairing** check, stable point/field encoding, gas schedule.
- Reference designs: Ethereum EIP-196/197-style ops; SP1 / Succinct EVM verifier layout; Solana BN254 verifier patterns.

**neo-zkvm work after natives land:**

1. Port SP1 Groth16 verifier to Neo contract.  
2. Bind `PublicInputs` (script_hash, input_hash, output_hash, gas, success) into field elements.  
3. On-chain VK storage keyed by `program_id` / vkey hash.  
4. Dual-run Path A fallback until Path B is audited.

### Path C — On-chain SNARK verify on **BLS12-381** (**if** Neo exposes BLS12-381 **pairings**)

```text
Custom / alternate wrapper: Groth16 or PLONK-KZG on BLS12-381
→ Neo multi-pairing check
```

- BLS12-381 is excellent for pairings (often preferred over BN254 for security).
- Requires **proof system on BLS12-381**, not stock “SP1 → BN254 EVM verifier” alone.
- Engineering cost: custom verifier circuit/export + Neo contract.

---

## 5. Recommendation for neo-zkvm (updated)

**Product direction:** Neo will add **BN254** support → **Path B is the target** trustless settlement (on-chain SP1 Groth16). Path A remains the **interim** path until BN254 pairings are live and the verifier contract is audited.

| Priority | Path | Status |
| --- | --- | --- |
| **P0 interim** | Path A (ECDSA N-of-M) | Implemented + T5 validated |
| **P0 target** | Path B (BN254 pairings + SP1 Groth16 verify on Neo) | **Planned** — Neo ships BN254 natives |
| **P1 optional** | Path A′ (BLS-sig attestors) | Spec’d below; ops improvement only |
| **P2 fallback** | Path C (BLS12-381 SNARK) | Only if BN254 unavailable |

**Do not** market “Neo has BLS ⇒ SP1 Groth16 verifies on-chain” without verifying:

1. Native is **pairing**, not only BLS-sig.  
2. Curve is **BN254** for stock SP1 EVM Groth16 (BLS12-381 alone is Path C).  
3. Gas allows full verify in one transaction.

---

## 6. Path A′ sketch (BLS-sig attestors)

If `CryptoLib` exposes e.g. `BlsVerify` / `BlsAggregateVerify` (names vary by version):

**Off-chain**

1. Same `attestation_digest` as Path A (or domain `neo-zkvm-attestation-bls-v1`).  
2. Each attestor produces a BLS signature on the digest.  
3. Aggregator combines signatures into one aggregate.

**On-chain**

1. Recompute digest.  
2. `BlsAggregateVerify(pubkeys[], message, agg_sig)` or equivalent.  
3. Same program_id / mode / nonce / app claim checks as Path A.

**Rust follow-up (not yet coded):** optional feature `bls-attestors` using `blst` or `bls12_381`, parallel to secp256r1 in `neo-zkvm-attestation`.

---

## 7. Path C research checklist (BLS12-381 SNARK on Neo)

When/if Neo documents multi-pairing:

- [ ] Confirm API: Miller loop / final exp / multi-pair check inputs layout  
- [ ] Gas cost for ~3 pairings (Groth16) or PLONK pair count  
- [ ] Choose proof system: Groth16-BLS12-381 vs PLONK-KZG-BLS12-381  
- [ ] Map neo-zkvm `PublicInputs` → field elements  
- [ ] Export VK + verifier contract in Neo C#/Python  
- [ ] Compare security/ops vs Path A multi-sig  
- [ ] **Do not** drop Path A until Path C is audited and mainnet-ready  

SP1-specific: confirm whether Succinct supports a **BLS12-381** wrapper for the same guest, or whether a recursive/other backend is required.

---

## 8. Summary table

| Question | Answer |
| --- | --- |
| Can BLS12-381 help zkVM settlement on Neo? | **Yes** — for **attestor aggregation** (A′) and, *if pairings exist*, for **on-chain SNARK** (C). |
| Does Neo BLS mean SP1 Groth16 verifies on-chain today? | **Usually no** — curve mismatch (BN254) and/or missing pairing native. |
| Most mature shippable stack now? | **NeoVM + SP1 off-chain + ECDSA N-of-M attestation (Path A)**. |
| Most trustless long-term (if natives allow)? | **On-chain multi-pairing** matching the proof curve (B or C). |

---

## References

- [neo-n3-attestation.md](neo-n3-attestation.md) — Path A ABI  
- [neo-n3-solution.md](neo-n3-solution.md) — product lock  
- [ROADMAP.md](ROADMAP.md) — phases including BLS/pairing  
- Neo CryptoLib framework docs: SHA256, VerifyWithECDsa, …  
- Succinct: bn254/bls12-381 **guest** precompiles vs EVM BN254 verify  
