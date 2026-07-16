# Neo zkVM Roadmap — Neo N3 settlement

## Goal

Ship a stack that:

1. Proves **NeoVM** execution (`neo-vm-rs`) with mature **SP1** proving.
2. Settles on Neo N3 by **deploying smart contracts** (and, where required, **consuming new CryptoLib natives** Neo ships).
3. **Target settlement (Path B):** on-chain **BN254** verify of SP1 Groth16/PLONK — same curve family as Ethereum / default SP1 EVM wrappers. **No ECDSA council** once pairings are live and gas is acceptable.
4. **Interim settlement (Path A):** N-of-M ECDSA attestation (implemented today) until BN254 pairings ship and the on-chain verifier is audited.

```text
Target (Path B — after Neo BN254 pairings)
──────────────────────────────────────────
NeoVM script + private args
        │
        ▼
neo-zkvm prove -m groth16   (SP1 → BN254)
        │
        ▼
Neo contract: BN254 multi-pairing + public inputs
        • store VK / program_id
        • verify proof
        • bind app claim, settle

Interim (Path A — live now)
───────────────────────────
SP1 verify off-chain → ECDSA N-of-M → NeoZkAttestation
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

- [x] CLI: `neo-zkvm attest` (`keygen`, `digest`, `sign`, `bundle`, `check`)
- [x] Config file for attestor keys / threshold / program_id (JSON)
- [x] Replay protection: random nonce helpers + contract nonce map + `IsNonceUsed`

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

### Phase 6 — Neo **BN254** natives + on-chain SP1 verify (Path B) — **target**

*Product decision: Neo will add **BN254** (not only BLS12-381). That matches default SP1 EVM Groth16.*

**6a — Neo / CryptoLib surface (upstream Neo)**

Minimum useful surface (Ethereum-compatible mental model):

- [ ] `Bn254Add` / `Bn254Mul` (G1) — or equivalent
- [ ] `Bn254Pairing` / multi-pairing check \(e(P_i,Q_i)\cdots=1\)
- [ ] Documented encoding: compressed vs uncompressed G1/G2, field element endianness
- [ ] Gas schedule published (target: full Groth16 verify practical in one tx)
- [ ] Negative tests: infinity, wrong subgroup, malformed points

**6b — neo-zkvm verifier contract (this repo)**

- [ ] Port SP1 BN254 Groth16 verifier (Succinct / EVM verifier layout) to Neo C# (or NeoVM script)
- [ ] Store verification key + `program_id` / vkey hash on-chain
- [ ] Map `PublicInputs` → BN254 scalar field elements (fixed layout)
- [ ] `SubmitProof(proof, public_inputs, app_claim…)` — no attestors
- [ ] Nonce / replay / app-claim binding (reuse Path A lessons)
- [ ] Gas benchmark on testnet vs Path A ECDSA N-of-M
- [ ] Keep Path A as fallback / dual-mode until Path B audited

**6c — Path C (BLS12-381 pairings only) — secondary**

- [ ] Only if BN254 slips; requires non-stock SP1→BN254 pipeline

See [neo-n3-crypto-surface.md](neo-n3-crypto-surface.md) for the decision matrix.

## Non-goals

- Claiming “Neo has BLS ⇒ SP1 Groth16 verifies on-chain” without pairing + curve match
- Changing Neo node / NeoVM unless Neo ships natives we only *consume*
- Using Mock proofs for value settlement
- Silent SP1 → Mock fallback on settlement path
