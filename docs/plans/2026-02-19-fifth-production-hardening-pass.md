# Fifth Production Hardening Pass Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the remaining policy gap by enforcing explicit open-source license compliance in local production gates and CI.

**Architecture:** Extend `cargo-deny` configuration with a strict SPDX allowlist and targeted clarification for one transitive crate lacking machine-readable metadata; then wire `licenses` checks into all production gate entrypoints.

**Tech Stack:** `cargo-deny`, `deny.toml`, GitHub Actions CI workflow, shell gate scripts.

### Task 1: Prove current license-policy gap (RED)

**Files:**
- Inspect: `.github/workflows/ci.yml`
- Inspect: `scripts/test.sh`
- Inspect: `scripts/verify-production.sh`

**Step 1: Verify license checks are missing before implementation**
- Run:
  - `rg -n "cargo deny --locked check licenses --show-stats" .github/workflows/ci.yml scripts/test.sh scripts/verify-production.sh`
- Expected before implementation: no matches.

### Task 2: Implement explicit deny license policy

**Files:**
- Modify: `deny.toml`

**Step 1: Add SPDX allowlist and private-crate behavior**
- Configure `[licenses]` with explicit allowed licenses used by current dependency graph.
- Set `[licenses.private] ignore = true` so unpublished workspace crates don't create false failures.

**Step 2: Add transitive crate clarification**
- Add `[[licenses.clarify]]` entry for `halo2` to map `COPYING` to `MIT OR Apache-2.0`.
- Use stable file hash so drift causes explicit review.

### Task 3: Enforce license check in all gates

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `scripts/test.sh`
- Modify: `scripts/verify-production.sh`
- Modify: `CONTRIBUTING.md`

**Step 1: CI audit job**
- Add `cargo deny --locked check licenses --show-stats` step after advisory/bans checks.

**Step 2: Local scripts parity**
- Expand scripts from 5-step to 6-step gate and add the same license command.

**Step 3: Contributor guidance**
- Document that license checks are required before PR.

### Task 4: Final verification (GREEN)

**Step 1: Policy command directly**
- Run: `cargo deny --locked check licenses --show-stats`
- Expected: `licenses ok: 0 errors`.

**Step 2: Full gate scripts**
- Run:
  - `bash scripts/test.sh`
  - `bash scripts/verify-production.sh`
- Expected: all steps pass, including license checks.
