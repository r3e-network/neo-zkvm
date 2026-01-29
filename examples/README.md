# Neo zkVM Examples

This directory contains example code demonstrating various features of Neo zkVM.

## Rust Examples

### basic.rs
Basic VM usage showing how to load and execute a simple script.
```bash
cargo run --example basic
```

### storage_example.rs
Demonstrates storage operations and Merkle proof support:
- Creating storage contexts
- CRUD operations (Create, Read, Update, Delete)
- Tracked storage with change logging
- Merkle root computation
- Prefix search functionality
- Read-only contexts

```bash
cargo run --example storage_example
```

### proof_generation.rs
Complete proof generation and verification workflow:
- Creating Neo VM scripts
- Preparing proof inputs with arguments
- Generating proofs (Mock/SP1/PLONK modes)
- Verifying proofs
- Analyzing public inputs

```bash
cargo run --example proof_generation
```

### native_contracts.rs
Using built-in native contracts (StdLib and CryptoLib):
- Serialization/deserialization
- Base64 encoding/decoding
- Number conversions (itoa/atoi)
- SHA256 and RIPEMD160 hashing
- NativeRegistry for unified access

```bash
cargo run --example native_contracts
```

## Assembly Examples (.neoasm)

### add.neoasm
Simple addition: `2 + 3 = 5`
```asm
PUSH2
PUSH3
ADD
RET
```

### multiply.neoasm
Simple multiplication example.

### compare.neoasm
Comparison operations example.

### loop.neoasm
Basic loop structure demonstration.

### fibonacci.neoasm
Calculates the Nth Fibonacci number using iteration.
- Demonstrates loop control flow
- Stack manipulation (OVER, SWAP, ROT)
- Conditional jumps (JMPIF)

### factorial.neoasm
Calculates N! (factorial) using iteration.
- Multiplication in loops
- Decrement and comparison
- Clean stack management

### complex_script.neoasm
Advanced example demonstrating:
- Array creation and manipulation
- Sum calculation over array elements
- Average computation
- Result packing

## OpCode Reference

Common opcodes used in examples:

| Category | OpCodes |
|----------|---------|
| Push | PUSH0-PUSH16, PUSHINT8/16/32/64 |
| Arithmetic | ADD, SUB, MUL, DIV, MOD, INC, DEC |
| Stack | DUP, DROP, SWAP, OVER, ROT, PICK |
| Comparison | LT, LE, GT, GE, NUMEQUAL |
| Control | JMP, JMPIF, JMPIFNOT, RET |
| Array | NEWARRAY, PICKITEM, SETITEM, SIZE |

## Running Examples

```bash
# Run Rust examples
cargo run --example basic
cargo run --example storage_example
cargo run --example proof_generation
cargo run --example native_contracts

# Assembly files can be assembled using neo-zkvm-cli
neo-zkvm assemble examples/fibonacci.neoasm
```

## Learn More

- [Neo VM Specification](https://docs.neo.org/docs/n3/reference/neo_vm)
- [Neo zkVM Documentation](../README.md)
