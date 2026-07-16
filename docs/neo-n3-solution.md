# Solution for current Neo N3

## Decision

**Target: today’s Neo N3 platform** — **no node/protocol changes**, only **deploy smart contracts**.

> Prove **NeoVM** (`neo-vm-rs`) with mature **SP1** off-chain.  
> **Today (interim):** settle with **N-of-M ECDSA** (Path A).  
> **Target:** Neo adds **BN254 pairings** → settle by **on-chain SP1 Groth16 verify** (Path B, no council).

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
| CLI operator tooling | ✅ `neo-zkvm attest` |
| Committee config JSON | ✅ program_id / magic / threshold / attestors |
| Neo N3 contract sketch | ✅ C# `NeoZkAttestation` (init-once, dup reject, LE) |
| Production attestor service | ⬜ Phase 3 |
| Audited mainnet deploy | ⬜ Phase 4 |

## Security rules

1. Settlement **rejects** Mock/Execute proof modes.
2. Attestors must **verify SP1** before signing.
3. Anyone can re-check SP1 off-chain against public data.
4. Contract enforces **program_id**, **network_magic**, **nonce**, **threshold**.

## Summary

**Interim stack (live today):**  
`neo-vm-rs` + SP1 Groth16 (off-chain verify) + `neo-zkvm-attestation` + Neo N-of-M ECDSA.

**Target stack (Neo ships BN254 pairings):**  
`neo-vm-rs` + SP1 Groth16 → **Neo on-chain BN254 verifier** (no attestor council).

**About BLS12-381:** still useful for BLS **signatures** (Path A′) or BLS12-381 SNARKs (Path C). Default SP1 EVM Groth16 is **BN254** — that is why Neo BN254 is the right product move. Details: [neo-n3-crypto-surface.md](neo-n3-crypto-surface.md).
