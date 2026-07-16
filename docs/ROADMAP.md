# Neo zkVM Roadmap — Neo N3 settlement (contracts only)

## Goal

Ship a stack that:

1. Proves **NeoVM** execution (`neo-vm-rs`) with mature **SP1** proving.
2. Settles on **current Neo N3** by **deploying smart contracts only** (no node/protocol changes).
3. Uses **N-of-M ECDSA attestations** on-chain because Neo N3 has no pairing precompile for cheap Groth16 verify.

```text
Off-chain                              On-chain Neo N3
─────────                              ───────────────
NeoVM script + private args
        │
        ▼
neo-zkvm prove (Mock dev | SP1/Groth16 prod)
        │
        ▼
verify_for_mode (SP1) ──► attestors sign digest
        │
        ▼
AttestationBundle ──────────────────► Contract:
                                      • check N-of-M ECDSA
                                      • check public inputs / app claim
                                      • mint / store / vote
```

## Phases

### Phase 0 — Product lock (done)

- [x] NeoVM semantics via `neo-vm-rs`
- [x] SP1 as proof backend
- [x] Docs: `neo-n3-solution.md`, attestation architecture

### Phase 1 — Attestation ABI + Rust library (this milestone)

- [x] Canonical attestation message + domain-separated digest
- [x] N-of-M sign / verify helpers (secp256r1, Neo-compatible story)
- [x] Unit tests + example binary
- [x] Neo N3 C# contract sketch (storage, threshold, submit)

### Phase 2 — Operator tooling

- [ ] CLI: `neo-zkvm attest` (build digest from proof file + sign)
- [ ] Config file for attestor keys / threshold / program_id
- [ ] Replay protection helpers (nonce registry docs)

### Phase 3 — Production SP1 operators

- [ ] Attestor service: verify SP1 Groth16 then sign
- [ ] Multi-attestor runbook + key ceremony notes
- [ ] Metrics / audit log of attested claims

### Phase 4 — Neo mainnet apps

- [ ] Production C# contract audit + deploy scripts
- [ ] First app (e.g. factors / membership settlement)
- [ ] Optional: optimistic challenge path (T3)

### Phase 5 — BLS-sig attestors (Path A′)

*Only when target Neo version documents BLS12-381 **signature** natives (not assumed on all N3 builds).*

- [ ] Inventory CryptoLib BLS methods per Neo version (`docs/neo-n3-crypto-surface.md`)
- [ ] Optional `neo-zkvm-attestation` feature: BLS aggregate sign/verify (e.g. `blst`)
- [ ] Neo contract variant: one aggregate BLS verify instead of N× ECDSA
- [ ] Gas comparison ECDSA N-of-M vs BLS aggregate

### Phase 6 — On-chain SNARK verify (Path B/C)

*Only if Neo exposes **pairings** (multi-pair check), not merely BLS signatures.*

- [ ] Confirm curve: **BN254** (Path B, matches default SP1 EVM Groth16) vs **BLS12-381** (Path C)
- [ ] If BN254 pairings: wire SP1 Groth16/PLONK verifier contract on Neo
- [ ] If BLS12-381 pairings only: evaluate BLS12-381 SNARK wrapper (not stock SP1→BN254)
- [ ] Public-input field encoding + VK storage + gas benchmarking
- [ ] Keep Path A as fallback until Path B/C is audited

See [neo-n3-crypto-surface.md](neo-n3-crypto-surface.md) for the decision matrix.

## Non-goals

- Claiming “Neo has BLS ⇒ SP1 Groth16 verifies on-chain” without pairing + curve match
- Changing Neo node / NeoVM unless Neo ships natives we only *consume*
- Using Mock proofs for value settlement
- Silent SP1 → Mock fallback on settlement path
