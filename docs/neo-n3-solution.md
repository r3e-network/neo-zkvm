# Solution for current Neo N3

## Decision

**Target: today’s Neo N3 platform** — **no node/protocol changes**, only **deploy smart contracts**.

> Prove **NeoVM** (`neo-vm-rs`) with mature **SP1** off-chain; settle on Neo with **N-of-M ECDSA attestations** (Neo has no pairing precompile for cheap Groth16).

```text
Off-chain (mature ZK)                    On-chain Neo N3 (contracts only)
─────────────────────                    ────────────────────────────────
NeoVM script + private args
        │
        ▼
neo-zkvm-program in SP1 (RISC-V guest)
runs neo_vm_guest::execute
        │
        ▼
SP1 prove → Groth16/PLONK proof
        │
        ▼
Attestors: verify_for_mode(Sp1/Groth16)
        then sign attestation digest  ──────►  NeoZkAttestation.Submit
                                               • VerifyWithECDsa × N
                                               • threshold
                                               • nonce replay guard
                                               • app claim / settle
```

| Layer | Choice | Why |
| --- | --- | --- |
| **Semantics** | NeoVM via `neo-vm-rs` | Current Neo N3 is NeoVM |
| **Proving** | SP1 (Groth16 preferred) | Most mature zk backend |
| **On-chain** | N-of-M ECDSA attestation | Deploy-only; uses SHA256 + VerifyWithECDsa |
| **Dev** | Mock / Execute | Never for value settlement |

Full ABI: [neo-n3-attestation.md](neo-n3-attestation.md)  
Roadmap: [ROADMAP.md](ROADMAP.md)  
Rust: `neo-zkvm-attestation`  
Contract sketch: `contracts/neo-n3/NeoZkAttestation/`

## What works today

| Capability | Status |
| --- | --- |
| NeoVM execute + public inputs | ✅ |
| SP1 / Mock prove-verify | ✅ |
| Canonical attestation digest + N-of-M sign/verify (Rust) | ✅ `neo-zkvm-attestation` |
| Example settlement flow | ✅ `zk_attestation_settlement` |
| Neo N3 contract sketch | ✅ C# `NeoZkAttestation` |
| Production attestor service | ⬜ Phase 3 |
| Audited mainnet deploy | ⬜ Phase 4 |

## Security rules

1. Settlement **rejects** Mock/Execute proof modes.
2. Attestors must **verify SP1** before signing.
3. Anyone can re-check SP1 off-chain against public data.
4. Contract enforces **program_id**, **network_magic**, **nonce**, **threshold**.

## Summary

**Best stack for current Neo N3 without system changes:**

`neo-vm-rs` + SP1 Groth16 (off-chain) + `neo-zkvm-attestation` + Neo contract N-of-M ECDSA.

**About BLS12-381 on Neo:** useful for **attestor aggregation** or future **pairing-based SNARK verify** only if CryptoLib exposes the right ops and the proof curve matches. Default SP1 EVM Groth16 is **BN254**. Details: [neo-n3-crypto-surface.md](neo-n3-crypto-surface.md).
