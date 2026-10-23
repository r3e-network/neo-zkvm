# SP1 v6 Migration Notes

Date: 2026-05-21

## Status

The workspace has been migrated from SP1 `5.2.x` to stable SP1 `6.2.1`.

Current SP1 dependencies:

- `sp1-sdk = { version = "6.2.1", default-features = false, features = ["blocking"] }`
- `sp1-zkvm = "6.2.1"`
- `sp1-build = "6.2.1"`

The host-side prover/verifier integration is optional behind the `sp1` feature. The default workspace build still supports execute and mock proofs without compiling the full SP1 proving stack or downloading SP1 proving artifacts.

## Design Decisions

### Default build

Default checks intentionally do not enable `sp1`.

Reasons:

- Unit tests, mock proofs, CLI smoke tests, VM execution, serialization, and verifier policy should be deterministic and offline-friendly.
- Ordinary `cargo test --workspace --locked` must not depend on a live SP1 artifact download.
- CI should catch core regressions quickly without requiring a Succinct toolchain.

### SP1 build

The SP1 integration is still linted and compiled in CI with:

```bash
SP1_FORCE_DUMMY=true cargo clippy -p neo-zkvm-prover -p neo-zkvm-verifier -p neo-zkvm-cli --features sp1 --locked -- -D warnings
```

This verifies the SP1 6.2.1 host API boundary while allowing CI to use a dummy ELF when the Succinct toolchain is not installed. Real production proving still requires a real guest ELF.

### Production proving

Real SP1 proofs require:

- `protoc`
- an installed Succinct/SP1 toolchain
- a built `neo-zkvm-program` guest ELF
- a release build for proving paths

For local production proving:

```bash
curl -L https://sp1.succinct.xyz | bash
sp1up
cargo build --release --features sp1
cargo run --release -p neo-zkvm-cli --features sp1 -- prove 12139E40 -m sp1
```

## API Changes Applied

- Switched host proof generation and verification to the SP1 6.2.1 blocking API.
- Imported the SP1 `ProveRequest` trait explicitly for `.core()`, `.compressed()`, `.plonk()`, and `.groth16()`.
- Retrieved verification keys via the SP1 `ProvingKey::verifying_key()` trait instead of relying on old public fields.
- Kept SP1 proof verification bound to:
  - proof mode
  - public input commitment
  - proof output hash
  - gas consumed
  - execution success flag
  - verification key hash

## Serialization Migration

The workspace direct dependency moved from `bincode` 1.x to `bincode` 2.0.1.

The shared codec wrapper in `neo-vm-guest` now provides:

- deterministic legacy-compatible bincode configuration
- explicit serialized output size checks
- trailing-byte rejection on decode

`bincode` 3.0.0 was checked during the dependency audit, but it is currently a compile-error placeholder and is not usable for this workspace.

## Security Advisory Posture

The all-feature dependency graph is accepted by `cargo deny` with explicit documented ignores for upstream advisories that cannot be fixed inside this workspace.

The lockfile updates `serial_test` to 3.5.0, removing the transitive `scc 2.4.0`
and RUSTSEC-2026-0205 from the all-feature graph.

Known accepted advisories:

- `RUSTSEC-2021-0139`: `ansi_term`, transitive through the optional SP1 tracing stack
- `RUSTSEC-2024-0436`: `paste`, transitive through the optional SP1 proving stack
- `RUSTSEC-2025-0119`: `number_prefix`, transitive through `indicatif` in SP1
- `RUSTSEC-2025-0134`: `rustls-pemfile`, transitive through `tonic` in SP1
- `RUSTSEC-2025-0141`: `bincode`, direct 2.0.1 plus SP1 transitive 1.3.3
- `RUSTSEC-2026-0002`: `lru`, transitive through SP1
- `RUSTSEC-2026-0173`: `proc-macro-error2`, transitive through SP1 JIT and `dynasm`

Risk boundary:

- Direct bincode usage is guarded by the workspace codec wrapper.
- SP1 advisories are confined to the optional `sp1` proving/verifying feature.
- `proc-macro-error2` is not invoked by neo-zkvm code and has no maintained
  compatible replacement in the pinned SP1 dependency graph.
- Replacement of SP1 transitive crates must come from upstream SP1 releases.

## Validation Commands

The migration was validated with:

```bash
cargo check --workspace --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
SP1_FORCE_DUMMY=true cargo clippy -p neo-zkvm-prover -p neo-zkvm-verifier -p neo-zkvm-cli --features sp1 --locked -- -D warnings
cargo deny --all-features --locked --offline check advisories --show-stats
cargo deny --all-features --locked --offline check bans sources --hide-inclusion-graph --show-stats
cargo deny --all-features --locked --offline check licenses --show-stats
cargo build -p neo-zkvm-prover --features sp1 --locked
cargo run --release -p neo-zkvm-cli --features sp1 -- prove 12139E40 -m sp1
```

## Operational Notes

- CI installs `protobuf-compiler` before the SP1 feature check because SP1 6.2.1 compiles protobuf-backed prover types.
- `SP1_FORCE_DUMMY=true` is for API compilation only. It must not be used as evidence of a real production proof.
- Release proof generation was validated with a real SP1 proof smoke command and should remain part of local production-readiness validation.
