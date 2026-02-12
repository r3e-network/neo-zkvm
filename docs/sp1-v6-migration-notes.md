# SP1 v6 Migration Notes

Date: 2026-02-12

## Purpose

This note captures the concrete findings from an SP1 `6.0.0-rc.1` compatibility attempt, why the workspace remains on SP1 `5.2.4`, and what must be true before retrying.

## Current Stable Baseline

- Workspace SP1 deps: `5.2.x`
  - `sp1-sdk = { version = "5.2", default-features = false }`
  - `sp1-zkvm = "5.2"`
  - `sp1-build = "5.2"`
- Prover/verifier hardening from review remains in place:
  - Output/public-input consistency checks
  - Full `ProofInput` hashing (non-lossy)
  - SP1 public-input parity fallback to mock
  - Faulted proof rejection semantics
- Validation status on this baseline:
  - `cargo fmt --all -- --check` passes
  - `cargo clippy --all --all-targets -- -D warnings` passes
  - `cargo test --all -q` passes
  - `cargo deny check advisories --show-stats` => `0 errors, 0 warnings`
  - Real CLI proof run in SP1 mode verifies successfully

## SP1 v6 (`6.0.0-rc.1`) Attempt Summary

### What compiled and worked

- Dependency resolution succeeded after bumping SP1 crates to `6.0.0-rc.1`.
- Code compiled after adapting to SP1 v6 API shifts.
- Tests and clippy were green once API/toolchain prerequisites were handled.

### Blocking/added operational requirements discovered

1. **`protoc` requirement surfaced immediately**
   - `sp1-prover-types` build script requires protobuf codegen.
   - Without `protoc`, build fails during dependency compilation.
   - Workaround used during investigation: local `protoc` binary via `PROTOC=$HOME/.local/protoc/bin/protoc`.

2. **Succinct toolchain/target compatibility sensitivity**
   - SP1 v6 `sp1-build` defaults to `riscv64im-succinct-zkvm-elf`.
   - Older succinct toolchain instances only had `riscv32im-succinct-zkvm-elf` and failed guest build.
   - Requires up-to-date toolchain where `rustc +succinct --print target-list` includes both (or at least required) target triples.

3. **SDK surface changes compared with v5**
   - Async/non-blocking API is default; existing code pattern needs blocking API or async refactor.
   - Blocking API path requires `sp1-sdk` feature `blocking`.
   - `setup` signature changed to accept `Elf` wrapper (`sp1_sdk::Elf::Static(...)`) in v6 paths.
   - `verify` signature includes explicit optional status parameter in blocking path.

### Security/advisory outcome on v6 RC

- `cargo deny check advisories` introduced a **new error** on the tested graph:
  - `RUSTSEC-2025-0134` (`rustls-pemfile` unmaintained)
  - transitively via `tonic` in `sp1-prover-types`.
- Net result: advisory posture became worse than current `5.2.4` baseline.

## Decision

Keep the production branch on SP1 `5.2.4` for now.

Reasoning:
- Lower environment friction (no mandatory `protoc` setup in current workflow).
- No new advisory error introduced.
- Existing hardening + full validation remains green.

## Retry Criteria for SP1 v6+

Only reattempt when all are true:

1. **Version maturity**
   - Prefer stable SP1 v6 release (not RC), unless a specific RC is required and accepted.

2. **Dependency risk check**
   - `cargo deny check advisories` must not introduce new errors compared to baseline.
   - If unavoidable advisories remain, they require explicit risk acceptance and documented rationale.

3. **Toolchain readiness**
   - `rustc +succinct --print target-list` includes required target(s) for that SP1 version.
   - `cargo prove` / `sp1up` version pinned and reproducible in CI/dev docs.

4. **Build environment readiness**
   - `protoc` availability standardized in CI + local setup docs (or removed upstream dependency).

5. **API compatibility completed**
   - Blocking vs async client choice made explicitly:
     - Option A: keep sync call sites with `sp1-sdk` `blocking` feature.
     - Option B: migrate prover/verifier integration to async end-to-end.

## Suggested Migration Procedure (next attempt)

1. Create a dedicated branch (e.g. `chore/sp1-v6-migration`).
2. Update SP1 crate versions in workspace and dependent crates.
3. Ensure environment prerequisites first (`sp1up` target support + `protoc`).
4. Apply SDK API migration edits (client setup/prove/verify signatures).
5. Run full gates:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all --all-targets -- -D warnings`
   - `cargo test --all -q`
   - `cargo deny check advisories --show-stats`
   - `cargo run -p neo-zkvm-cli --release -- prove 12139E40 -m sp1 --allow-fallback`
6. Compare against baseline and decide go/no-go.

## Rollback Plan

If migration regresses advisories/stability:

- Revert SP1 versions back to `5.2.x`.
- Regenerate lockfile (`cargo update`).
- Re-run full validation gates to confirm baseline health.

