# Neo zkVM vs Ethereum-style zkVMs

This guide explains **what Neo zkVM does**, **how Ethereum zkVMs (SP1, RISC Zero, zkEVM) work**, and how the examples in this repo map to those patterns.

## What is a zkVM?

A **zero-knowledge virtual machine** proves that a program executed correctly:

1. A **guest program** (or bytecode) runs on deterministic semantics.
2. **Private inputs** (witness) are used during execution but not revealed.
3. A **proof** attests to correct execution and to **public outputs** (and often public input hashes).
4. Anyone can **verify** the proof cheaply without re-running the full computation or seeing secrets.

```text
                    private inputs (witness)
                              |
                              v
  [ program / bytecode ] --> execute --> public outputs + proof
                              |
                              v
                    verifier (on-chain or off-chain)
```

## How Ethereum zkVMs work

### General-purpose zkVMs (SP1, RISC Zero)

| Piece | Role |
| --- | --- |
| **ISA** | Usually RISC-V; you write **Rust** (or other languages) that compile to the guest ISA |
| **Host** | Loads guest ELF, supplies private/public inputs, runs prover |
| **Guest** | Reads inputs, computes, **commits public values** into the proof |
| **Proof systems** | STARK-style recursion + often **Groth16/PLONK** wrappers for cheap EVM verify |
| **On-chain** | Solidity verifier checks succinct proof (~hundreds of k gas for Groth16) |

**Typical Ethereum use cases:**

| Use case | What you prove | Private | Public |
| --- | --- | --- | --- |
| **Hello / factors** | \(n = p \times q\) for composite \(n\) | \(p, q\) | \(n\) (or product on stack) |
| **Range / threshold** | balance ≥ min | balance | min, yes/no |
| **Preimage** | \(\mathrm{hash}(x) = H\) | \(x\) | \(H\) |
| **Membership** | \(x \in S\) or \(\mathrm{hash}(x) \in\) allowlist | \(x\) | set / root |
| **Selective disclosure** | field \(f\) of private record equals \(v\) | full record | \(f, v\) or hash |
| **Merkle inclusion** | leaf is under root | leaf, path | root |
| **Rollup / batch** | many txs → new state root | txs | roots, commitments |
| **Light client / coprocessor** | historical chain facts | headers/logs | claim |
| **zkEVM** | full EVM block is valid | block body | state roots |

SP1 and RISC Zero prove **arbitrary Rust**; zkEVMs prove **Ethereum L1/L2 execution** specifically.

### zkEVMs (Scroll, Polygon zkEVM, Linea, …)

A zkEVM re-implements (or circuits) the **EVM** so each L2 block gets a validity proof. That is a specialized zkVM-like system for Ethereum opcodes and gas, not a general RISC-V guest.

## Product stance for current Neo N3

For **today’s Neo N3**, the intended solution is documented in [neo-n3-solution.md](neo-n3-solution.md):

- **Prove NeoVM** (not drop NeoVM for pure RISC-V apps).
- Use **SP1** as the mature **proof backend** (RISC-V guest runs the Neo interpreter).

Pure SP1 “any Rust program” is mature for generic apps but does **not** by itself mean NeoVM/N3 script fidelity.

## What Neo zkVM can do

Neo zkVM is a **Neo N3-oriented zk stack**:

| Capability | Details |
| --- | --- |
| **ISA** | Canonical **NeoVM** opcodes via shared **`neo-vm-rs`** (not a second engine) |
| **Guest** | `neo-vm-guest::execute` + deterministic syscalls (`SHA256`, `RIPEMD160`, `Hash160`, `Hash256`) |
| **Proving** | `NeoProver`: `Execute` / `Mock` (dev) / **SP1** compressed / Plonk / Groth16 (production) |
| **Verification** | Mode-pinned `verify_for_mode` / `verify_with_vkey` (never trust bare mode alone in production) |
| **Public binding** | `PublicInputs`: script hash, input hash, output hash, gas, success |
| **Tooling** | CLI: run, prove, verify, asm, disasm, debug, inspect |
| **Safety** | Runtime gas (1 / instruction), size limits, fail-closed SP1 fallback |

**What you prove today:** correct execution of a **NeoVM script** on private **stack arguments**, producing a public **result** and hashes. With **Mock**, demos are local-only (forgeable). With **SP1/Plonk/Groth16**, proofs are cryptographically sound (needs toolchain + real ELF).

**Not the same as a zkEVM:** Neo zkVM does not re-execute Ethereum blocks. It re-uses SP1 as the **proof backend** for Neo bytecode semantics.

## Mapping Ethereum examples → Neo zkVM

| Ethereum / RISC Zero / SP1 pattern | Neo zkVM example binary | Idea |
| --- | --- | --- |
| Hello world / composite factors | `zk_factors` | Private \(p,q\); public product |
| Range proof / balance threshold | `zk_range` | Private balance ≥ public min |
| Hash preimage | `zk_preimage` | Private password → public hash |
| Set membership / allowlist | `zk_membership` | Private value hashes to allowed digest |
| Selective disclosure (JSON field) | `zk_selective_disclosure` | Private payload; prove one field |
| Merkle inclusion | `zk_merkle_inclusion` | Leaf + path → root |
| Batch / rollup | `zk_dex_rollup`, `batch_verification` | Many ops → one proof |
| DAO / governance | `zk_dao_voting` | Eligibility-style predicate |
| Off-chain compute | `zk_scaling` | Heavy loop off-chain, verify once |

## Security model (shared with Ethereum zkVMs)

1. **Pin proof mode** at verify time (`ProofMode::Groth16`, not `proof.proof_mode`).
2. **Mock/Execute are not ZK** — fine for tests and DX only.
3. Bind **public claims** (expected hash, threshold, root) in the verifier, not only “proof verifies”.
4. Only **deterministic** syscalls belong in the guest (no wall-clock, no host storage).

## Further reading

- [Getting Started](getting-started.md)
- [Architecture](architecture.md)
- [SP1 v6 migration](sp1-v6-migration-notes.md)
- SP1: <https://github.com/succinctlabs/sp1>
- RISC Zero examples: <https://dev.risczero.com/api/zkvm/examples>
