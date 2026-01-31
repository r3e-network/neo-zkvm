# Neo zkVM Examples

This directory contains working examples demonstrating various features of Neo zkVM.

## Available Examples

### Basic Examples

| Example | Description | Command |
|---------|-------------|---------|
| `basic` | Simple 2 + 3 = 5 execution | `cargo run --example basic` |
| `vm_examples` | Comprehensive VM feature demo | `cargo run --example vm_examples` |

### DeFi & Smart Contract Examples

| Example | Description | Command |
|---------|-------------|---------|
| `token_contract` | NEP-17 compatible token with transfers | `cargo run --example token_contract` |
| `multisig_wallet` | 2-of-3 multi-signature wallet | `cargo run --example multisig_wallet` |
| `amm_swap` | Constant product AMM swap logic | `cargo run --example amm_swap` |
| `native_contracts` | StdLib and CryptoLib usage | `cargo run --example native_contracts` |
| `storage_example` | Key-value storage with Merkle proofs | `cargo run --example storage_example` |
| `proof_generation` | ZK proof generation and verification | `cargo run --example proof_generation` |

## Quick Start

```bash
# Run the basic example
cargo run --example basic

# Run the token contract example
cargo run --example token_contract

# Run the AMM swap example
cargo run --example amm_swap
```

## Example Descriptions

### Token Contract (`token_contract.rs`)

Demonstrates a NEP-17 compatible token implementation:
- Token initialization with metadata
- Balance tracking
- Transfer operations
- VM-based validation
- State verification with Merkle proofs
- Native contract integration

### Multi-Signature Wallet (`multisig_wallet.rs`)

Shows a 2-of-3 multi-signature wallet:
- Signature threshold verification
- VM-based approval logic
- Transfer execution
- Failed attempt handling

### AMM Swap (`amm_swap.rs`)

Implements constant product AMM swap logic:
- Liquidity pool initialization
- Constant product formula (x * y = k)
- Swap amount calculation with fees
- Price impact analysis
- Slippage protection
- VM verification of calculations

## Creating Your Own Example

1. Create a new file in `crates/neo-vm-core/examples/`
2. Add a descriptive comment at the top
3. Use `cargo run --example <name>` to test
4. Update this README

## Example Template

```rust
//! Example Name
//!
//! Brief description of what this example demonstrates.

use neo_vm_core::{NeoVM, VMState, StackItem};

fn main() {
    println!("=== Example Name ===\n");
    
    // Your example code here
    
    println!("\n=== Example Complete ===");
}
```
