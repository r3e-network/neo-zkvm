# Migration Guide

## v0.1.0 to v0.2.0

This guide outlines breaking changes and migration steps for upgrading from neo-zkvm v0.1.0 to v0.2.0.

## Breaking Changes

### 1. `load_script` API Change

**Before (v0.1.0):**
```rust
let mut vm = NeoVM::new(1_000_000);
vm.load_script(script); // Returns ()
```

**After (v0.2.0):**
```rust
let mut vm = NeoVM::new(1_000_000);
vm.load_script(script)?; // Returns Result<(), VMError>
```

**Migration:**
Add error handling for script loading failures:
```rust
if let Err(e) = vm.load_script(script) {
    eprintln!("Failed to load script: {}", e);
    return;
}
```

### 2. `VMError` Enum Changes

**New error variants:**
- `VMError::StackOverflow` - Stack depth exceeded maximum
- `VMError::InvalidScript` - Script validation failed

**Migration:**
Update pattern matching to handle new variants:
```rust
match vm.load_script(script) {
    Ok(()) => { /* success */ }
    Err(VMError::InvalidScript) => { /* handle oversized script */ }
    Err(e) => { /* handle other errors */ }
}
```

### 3. Script Size Limit

Maximum script size is now 1MB. Scripts exceeding this limit will return `VMError::InvalidScript`.

**Migration:**
Validate script size before loading:
```rust
if script.len() > 1024 * 1024 {
    return Err("Script too large".into());
}
vm.load_script(script)?;
```

### 4. `ProofOutput` Structure

**New field:**
```rust
pub struct ProofOutput {
    pub state: u8,
    pub result: Option<StackItem>,
    pub gas_consumed: u64,
    pub error: Option<String>, // NEW FIELD
}
```

**Migration:**
Access error information when needed:
```rust
let output = execute(input);
if let Some(err) = &output.error {
    eprintln!("Execution error: {}", err);
}
```

### 5. StackItem Send/Sync

`StackItem` and related types are now `Send + Sync`.

**Before:**
Types were not thread-safe.

**After:**
Safe to share across threads with proper synchronization.

## New Features

### 1. SP1 zkVM Integration

Build the guest program to enable full proof generation:
```bash
cargo build --package neo-zkvm-program --release
```

### 2. Opcode Handler Pattern

New handler-based architecture for opcode implementation:
```rust
use neo_vm_core::engine::handlers::{OpHandler, arithmetic_handlers::AddHandler};

let handler = AddHandler;
handler.handle(&mut vm, &mut ctx)?;
```

### 3. Enhanced CI/CD

New security and coverage checks:
```bash
cargo deny check advisories  # Security audit
cargo tarpaulin --out Xml    # Code coverage
```

## Deprecations

None in this release.

## Performance Notes

- Release builds now use LTO and optimized codegen-units
- Performance improvements in Merkle tree computation (iterative vs recursive)
- Better memory allocation patterns in tracing

## Testing Recommendations

Run comprehensive tests after migration:
```bash
cargo test --all
cargo clippy --all -- -D warnings
cargo fmt --all -- --check
```
