# neo-zkvm-examples

<!-- N4-CRATE-VISUAL-GUIDE:START -->

## Crate Visual Learning Guide

These diagrams are local to this crate. They explain `neo-zkvm-examples` as an independent unit: where it sits in the Neo N4 stack, which boundary it owns, how its internal workflow runs, and how data moves through it.

| View | Diagram | Source |
| --- | --- | --- |
| Position in Neo N4 | ![Position](docs/figures/position.svg) | [Mermaid](docs/figures/position.mmd) |
| Technical principles | ![Principles](docs/figures/principles.svg) | [Mermaid](docs/figures/principles.mmd) |
| Architecture | ![Architecture](docs/figures/architecture.svg) | [Mermaid](docs/figures/architecture.mmd) |
| Workflow | ![Workflow](docs/figures/workflow.svg) | [Mermaid](docs/figures/workflow.mmd) |
| Dataflow | ![Dataflow](docs/figures/dataflow.svg) | [Mermaid](docs/figures/dataflow.mmd) |

### Role in Neo N4

- **Layer:** Neo zkVM stack
- **Purpose:** Runnable examples that demonstrate common proof flows and application patterns.
- **Primary inputs:** sample script, example input, local prover
- **Primary outputs:** example proof, tutorial output, regression sample
- **Downstream consumers:** zkVM users, L2 prover service, L1 verification adapter

### Boundary and Responsibilities

- **Owns:** Demonstrate APIs, Exercise edge cases, Document expected outputs
- **Consumes:** sample script, example input, local prover
- **Produces:** example proof, tutorial output, regression sample
- **Used by:** zkVM users, L2 prover service, L1 verification adapter

### Learning Path

1. Start with the position diagram to understand why this crate exists and who calls it.
2. Read the technical principles diagram to identify the invariants and responsibility boundary.
3. Use the architecture diagram to connect public inputs, internal components, dependencies, and outputs.
4. Follow the workflow and dataflow diagrams before reading source files or tests.

<!-- N4-CRATE-VISUAL-GUIDE:END -->
