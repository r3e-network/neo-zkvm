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

### Phase 5 — Stretch (requires Neo crypto upgrades or heavy gas)

- [ ] Native pairing / Groth16 precompile on Neo (protocol change — out of scope unless Neo adds it)
- [ ] Full on-chain SNARK verify without attestors

## Non-goals

- Changing Neo node / NeoVM / native syscalls
- Using Mock proofs for value settlement
- Silent SP1 → Mock fallback on settlement path
