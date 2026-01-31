# Neo zkVM Examples

This directory contains working examples demonstrating various features of Neo zkVM.

## Available Examples

### 1. Basic (`basic.rs`)
Simple VM usage - demonstrates 2 + 3 = 5.

```bash
cargo run --example basic
```

### 2. Native Contracts (`native_contracts.rs`)
Shows how to use StdLib and CryptoLib native contracts.

```bash
cargo run --example native_contracts
```

### 3. Storage (`storage_example.rs`)
Demonstrates key-value storage with Merkle proofs.

```bash
cargo run --example storage_example
```

### 4. Proof Generation (`proof_generation.rs`)
Shows how to generate and verify ZK proofs.

```bash
cargo run --example proof_generation
```

## Running All Examples

```bash
./scripts/run-examples.sh
```

Or manually:

```bash
cargo run --example basic
cargo run --example native_contracts
cargo run --example storage_example
cargo run --example proof_generation
```

## Creating Your Own Example

1. Create a new file in `examples/` directory
2. Add it to `Cargo.toml` in the `[[example]]` section
3. Run with `cargo run --example <name>`

## Example Structure

Each example should:
- Have a descriptive comment at the top
- Use `?` for error handling where possible
- Print clear output
- Demonstrate a specific feature
