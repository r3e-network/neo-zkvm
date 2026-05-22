# neo-vm-guest

<!-- N4-CRATE-VISUAL-GUIDE:START -->

## Crate Visual Learning Guide

These diagrams are local to this crate. They explain `neo-vm-guest` as an independent unit: where it sits in the Neo N4 stack, which boundary it owns, how its internal workflow runs, and how data moves through it.

| View | Diagram | Source |
| --- | --- | --- |
| Position in Neo N4 | ![Position](docs/figures/position.svg) | [Mermaid](docs/figures/position.mmd) |
| Technical principles | ![Principles](docs/figures/principles.svg) | [Mermaid](docs/figures/principles.mmd) |
| Architecture | ![Architecture](docs/figures/architecture.svg) | [Mermaid](docs/figures/architecture.mmd) |
| Workflow | ![Workflow](docs/figures/workflow.svg) | [Mermaid](docs/figures/workflow.mmd) |
| Dataflow | ![Dataflow](docs/figures/dataflow.svg) | [Mermaid](docs/figures/dataflow.mmd) |

### Role in Neo N4

- **Layer:** zkVM guest facade
- **Purpose:** Guest-facing adapter that exposes shared NeoVM execution APIs in zkVM-compatible form.
- **Primary inputs:** guest bytecode, stack input, shared VM crate
- **Primary outputs:** guest execution result, public output seed, fault reason
- **Downstream consumers:** neo-zkvm-program, neo-zkvm-prover, examples

### Boundary and Responsibilities

- **Owns:** Call shared VM, Keep guest ABI small, Return deterministic result
- **Consumes:** guest bytecode, stack input, shared VM crate
- **Produces:** guest execution result, public output seed, fault reason
- **Used by:** neo-zkvm-program, neo-zkvm-prover, examples

### Learning Path

1. Start with the position diagram to understand why this crate exists and who calls it.
2. Read the technical principles diagram to identify the invariants and responsibility boundary.
3. Use the architecture diagram to connect public inputs, internal components, dependencies, and outputs.
4. Follow the workflow and dataflow diagrams before reading source files or tests.

<!-- N4-CRATE-VISUAL-GUIDE:END -->
