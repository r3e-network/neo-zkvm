# Neo zkVM Figures

These diagrams provide a visual companion to the Neo zkVM implementation. They are generated from `docs/figures/generate_figures.py` so the English and Chinese versions stay aligned.

## Figure Set

| Figure | Purpose |
| --- | --- |
| [Architecture](neo-zkvm-architecture.svg) | Crate boundaries, SP1 integration, verifier policy, and the shared `neo-vm-rs` execution core. |
| [Dataflow](neo-zkvm-dataflow.svg) | How a script becomes `ProofInput`, `ProofOutput`, `PublicInputs`, proof bytes, and verifier evidence. |
| [Workflow](neo-zkvm-workflow.svg) | CLI and library paths for `run`, `prove`, and verifier operations. |
| [Proof Objects](neo-zkvm-proof-objects.svg) | The proof ABI structures and how they bind to `neo-vm-rs` types. |
| [Verification Logic](neo-zkvm-verification.svg) | Fail-closed verifier checks for Execute, Mock, SP1, PLONK, and Groth16 modes. |
| [Mathematical Design](neo-zkvm-math.svg) | Hashes, commitments, public inputs, and acceptance conditions. |

## Preview

![Neo zkVM Architecture](neo-zkvm-architecture.svg)

## Other Languages

- [中文图表](../zh/figures/README.md)
