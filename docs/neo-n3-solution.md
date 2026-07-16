# Solution for current Neo N3

## Decision

**Target: today’s Neo N3 platform** (NeoVM contracts, Neo stack values, shared `neo-vm-rs`).

The product is **not** “replace NeoVM with a pure RISC-V app zkVM.”  
It is:

> **Prove NeoVM execution that matches Neo N3 semantics, using the most mature proof backend available (SP1).**

```text
  Neo N3 world                         zk world
  ────────────                         ────────
  NeoVM scripts / args                 SP1 (RISC-V guest ELF)
  neo-vm-rs semantics         ◄──────  neo-zkvm-program runs
  same opcodes & StackItem              neo_vm_guest::execute
           │                                    │
           │         public inputs + proof      │
           └──────── verify_for_mode ───────────┘
                    (off-chain now; on-chain later)
```

| Layer | Choice | Why |
| --- | --- | --- |
| **What is proven** | **NeoVM** (`neo-vm-rs`) | Current Neo N3 contracts/scripts are NeoVM, not RISC-V apps |
| **How it is proven** | **SP1** (Plonk/Groth16 for cheap verify) | Most mature general zk proving stack we integrate |
| **Dev modes** | Execute / Mock | Local DX; **not** production ZK |
| **Single semantics** | One engine: `neo-vm-rs` | No second VM that drifts from Neo N3 |

Pure “Rust → RISC-V only” (no NeoVM) is fine for generic apps, but it does **not** prove “this Neo N3 script ran.” For **current Neo N3**, NeoVM-in-the-guest is required.

---

## What works today (this repo)

| Capability | Status |
| --- | --- |
| Execute NeoVM scripts with shared `neo-vm-rs` | ✅ |
| Private stack arguments (witness) | ✅ |
| Deterministic crypto syscalls (SHA256, Hash160, Hash256, RIPEMD160) | ✅ |
| Public input binding (script/input/output hash, gas, success) | ✅ |
| Mock prove/verify for tests & examples | ✅ |
| SP1 host path (feature `sp1`, real ELF when toolchain present) | ✅ (env-dependent) |
| Mode-pinned verification (no silent Mock accept) | ✅ |
| CLI: run / prove / verify / asm / disasm / debug | ✅ |
| Ethereum-style *application patterns* on Neo scripts (factors, range, …) | ✅ examples |

## What “production on Neo N3” still needs (platform work)

These are **integration** items, not a change of zkVM family:

1. **On-chain (or Neo-native) verifier**  
   Deploy/verify SP1 Groth16/PLONK (or a Neo-friendly wrapper) so a Neo N3 contract can check `NeoProof` public values.

2. **Public-value ↔ Neo app ABI**  
   Map `PublicInputs` + app results to storage updates, NEP-17, votes, etc.

3. **Syscall policy for mainnet**  
   Only deterministic adapters in-guest; chain state as **explicit inputs/commitments** (not live RPC inside the proof).

4. **NEF / contract packaging** (optional)  
   Prove raw scripts today; full NEF parse can be added when packaging needs it.

5. **Version pin**  
   `neo-vm-rs` rev must track the Neo N3 node/tooling revision you ship against.

---

## How a Neo N3 app uses it

```text
1. Build NeoVM script (asm / compiler / existing contract fragment)
2. Private args = user secrets (password, factors, balance, …)
3. neo-zkvm prove  (Mock local | SP1/Groth16 production)
4. Submit proof + public claims to your Neo N3 contract/service
5. Contract: verify_for_mode(..., Groth16) + check public result/root/hash
6. Then mint / transfer / record vote / update root
```

**Security rules (same as Ethereum zk apps):**

- Production: **Sp1 / Plonk / Groth16 only**, never Mock.  
- Always pin mode: `verify_for_mode(&proof, ProofMode::Groth16)`.  
- Always check the **application claim** (e.g. `result == expected_n`), not only “proof valid.”

---

## Relation to “most mature zkVM”

| Question | Answer |
| --- | --- |
| Do we use mature SP1? | **Yes** — as the **proving engine**. |
| Is the guest pure arbitrary RISC-V app code? | **No as the default product** — the guest’s job is to run **NeoVM**. |
| Why? | Current Neo N3 **is** NeoVM; proofs must mean NeoVM if they settle Neo contracts. |

That is the same pattern as “run Reth/zkEVM client inside SP1”: mature RISC-V zkVM **proves another machine**.

---

## Non-goals (unless product expands later)

- Replacing NeoVM with a separate RISC-V contract language for Neo mainnet  
- Proving full Neo consensus / full node (out of scope)  
- Host-dependent syscalls inside proofs (time, storage, network)

---

## Summary

**For current Neo N3, the solution is this stack:**

1. **Semantics:** `neo-vm-rs` (NeoVM)  
2. **Proofs:** SP1 (mature backend)  
3. **Product:** prove Neo scripts + private args → public outputs + proof  
4. **Platform:** Neo contracts verify and act on those public outputs  

That **does** work for NeoVM and for Neo N3; pure RISC-V-only guests do not replace step 1 for current N3.
