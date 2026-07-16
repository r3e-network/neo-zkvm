#!/bin/bash
# Validate version/metadata consistency before a release.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

VERSION="$(
  sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1
)"
[ -n "$VERSION" ] || fail "Could not read workspace version from Cargo.toml"

echo "Workspace version: $VERSION"

# Workspace member package versions must inherit or match.
while IFS= read -r crate_toml; do
  if grep -q 'version.workspace = true' "$crate_toml"; then
    continue
  fi
  crate_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$crate_toml" | head -1 || true)"
  if [ -n "${crate_version:-}" ] && [ "$crate_version" != "$VERSION" ]; then
    fail "$crate_toml version $crate_version != workspace $VERSION"
  fi
done < <(find crates -name Cargo.toml -print | sort)

# Workspace path dependency versions should match the release version.
for dep in neo-vm-guest neo-zkvm-prover neo-zkvm-verifier neo-zkvm-attestation; do
  if ! grep -E "^\s*${dep}\s*=\s*\{[^}]*version\s*=\s*\"${VERSION}\"" Cargo.toml >/dev/null; then
    fail "workspace.dependencies.${dep} version is not ${VERSION}"
  fi
done

# CHANGELOG must mention this version (or Unreleased for pre-release work).
if ! grep -qE "^## \[(Unreleased|${VERSION})\]" CHANGELOG.md; then
  fail "CHANGELOG.md has no section for Unreleased or ${VERSION}"
fi

# Ensure package descriptions exist for publishable crates.
for crate in neo-vm-guest neo-zkvm-prover neo-zkvm-verifier neo-zkvm-attestation neo-zkvm-cli neo-zkvm-program; do
  desc="$(sed -n 's/^description = "\(.*\)"/\1/p' "crates/${crate}/Cargo.toml" | head -1 || true)"
  [ -n "${desc:-}" ] || fail "crates/${crate}/Cargo.toml missing description"
done

# neo-zkvm-examples must stay unpublished.
if ! grep -q 'publish = false' crates/neo-zkvm-examples/Cargo.toml; then
  fail "neo-zkvm-examples must set publish = false"
fi

echo "Release metadata checks passed for v${VERSION}"
