# Neo zkVM CLI

A comprehensive command-line toolkit for Neo zkVM development.

## Installation

```bash
cargo install --path crates/neo-zkvm-cli
```

Or build from source:

```bash
cargo build --release -p neo-zkvm-cli
```

## Commands

### run

Execute a script and display results.

```bash
neo-zkvm run <script> [--gas <limit>]
```

**Examples:**

```bash
# Execute hex bytecode (PUSH2 PUSH3 ADD RET)
neo-zkvm run 12139E40

# Execute from binary file
neo-zkvm run script.bin

# With custom gas limit
neo-zkvm run 12139E40 --gas 500000
```

**Output:**

```
═══════════════════════════════════════
  EXECUTION RESULT
═══════════════════════════════════════
  State:        Halt
  Gas consumed: 12
  Stack depth:  1
───────────────────────────────────────
  Stack (top → bottom):
    [0] Integer(5)
═══════════════════════════════════════
```

### prove

Generate a ZK proof for script execution.

```bash
neo-zkvm prove <script> [--proof-mode <mode>|-m <mode>] [--allow-fallback] [--gas <limit>]
```

**Options:**

| Flag               | Alias | Default | Description                                              |
| ------------------ | ----- | ------- | -------------------------------------------------------- |
| `--proof-mode`     | `-m`  | `sp1`   | Proof mode: `execute`, `mock`, `sp1`, `plonk`, `groth16` |
| `--allow-fallback` |       | off     | Allow silent downgrade to mock if SP1 unavailable        |
| `--gas`            | `-g`  | 1000000 | Gas limit                                                |

**Examples:**

```bash
# Default SP1 proof
neo-zkvm prove 12139E40

# Mock proof for fast testing
neo-zkvm prove 12139E40 -m mock

# Groth16 proof (smallest, on-chain)
neo-zkvm prove 12139E40 --proof-mode groth16

# SP1 with fallback to mock if toolchain missing
neo-zkvm prove 12139E40 -m sp1 --allow-fallback
```

### asm

Assemble source code to bytecode.

```bash
neo-zkvm asm <source>
```

**Examples:**

```bash
# Inline assembly
neo-zkvm asm "PUSH2 PUSH3 ADD RET"

# From file
neo-zkvm asm program.neoasm
```

**Assembly Syntax:**

```asm
; Comments start with semicolon
; Labels end with colon

start:
    PUSH2           ; Push constant 2
    PUSH3           ; Push constant 3
    ADD             ; Add top two values
    JMP end         ; Jump to label

loop:
    DUP
    DEC
    JMPIF loop      ; Loop while non-zero

end:
    RET             ; Return
```

**Supported Syntax Sugar:**

| Sugar                 | Expansion                             |
| --------------------- | ------------------------------------- |
| `PUSH <n>`            | Auto-selects optimal PUSH instruction |
| `INC2`, `INC3`, ...   | Multiple INC instructions             |
| `DEC2`, `DEC3`, ...   | Multiple DEC instructions             |
| `DUP2`, `DUP3`, ...   | Multiple DUP instructions             |
| `DROP2`, `DROP3`, ... | Multiple DROP instructions            |
| `TRUE`                | Alias for PUSH1                       |
| `FALSE`               | Alias for PUSH0                       |
| `NEG`                 | Alias for NEGATE                      |

**Macro Support:**

```asm
; Define a macro
.macro double
    DUP
    ADD
.endmacro

; Use the macro
PUSH5
%double         ; Expands to: DUP ADD
RET
```

**Macros with Parameters:**

```asm
.macro add_n $n
    PUSH $n
    ADD
.endmacro

PUSH10
%add_n 5        ; Expands to: PUSH5 ADD
RET
```

### disasm

Disassemble bytecode to readable format.

```bash
neo-zkvm disasm <hex>
```

**Examples:**

```bash
neo-zkvm disasm 12139E40
neo-zkvm disasm script.bin
```

**Output:**

```
0000:  12                PUSH2
0001:  13                PUSH3
0002:  9E                ADD
0003:  40                RET
```

### debug

Trace script execution with the shared `neo-vm-rs` interpreter semantics.

```bash
neo-zkvm debug <script>
```

**Example Output:**

```
$ neo-zkvm debug 12139E40
Tracing script with shared neo-vm-rs semantics...

Executed instructions:
  0x0000: 12  PUSH2
  0x0001: 13  PUSH3
  0x0002: 9E  ADD
  0x0003: 40  RET

Execution result:
  State: Halt
  Stack: [Integer(5)]
```

### inspect

Analyze and display detailed script information.

```bash
neo-zkvm inspect <script>
```

**Output includes:**

- Script size and hash
- Opcode statistics
- Jump targets
- Gas estimation (min/max)
- Full disassembly

**Example:**

```bash
neo-zkvm inspect 12139E40
```

## Input Formats

The CLI accepts scripts in multiple formats:

| Format          | Example          | Description                |
| --------------- | ---------------- | -------------------------- |
| Hex string      | `12139E40`       | Raw hex bytecode           |
| Hex with prefix | `0x12139E40`     | Hex with 0x prefix         |
| Binary file     | `script.bin`     | Binary file                |
| NEF file        | `contract.nef`   | Neo Executable Format      |
| Assembly file   | `program.neoasm` | Assembly source (asm only) |

## Opcode Reference

### Constants (0x00-0x20)

| Opcode       | Hex       | Description                |
| ------------ | --------- | -------------------------- |
| PUSHINT8     | 0x00      | Push 1-byte signed integer |
| PUSHINT16    | 0x01      | Push 2-byte signed integer |
| PUSHINT32    | 0x02      | Push 4-byte signed integer |
| PUSHINT64    | 0x03      | Push 8-byte signed integer |
| PUSHNULL     | 0x0B      | Push null                  |
| PUSHDATA1    | 0x0C      | Push data (1-byte length)  |
| PUSHDATA2    | 0x0D      | Push data (2-byte length)  |
| PUSHM1       | 0x0F      | Push -1                    |
| PUSH0-PUSH16 | 0x10-0x20 | Push 0-16                  |

### Flow Control (0x21-0x41)

| Opcode   | Hex  | Description        |
| -------- | ---- | ------------------ |
| NOP      | 0x21 | No operation       |
| JMP      | 0x22 | Unconditional jump |
| JMPIF    | 0x24 | Jump if true       |
| JMPIFNOT | 0x26 | Jump if false      |
| CALL     | 0x34 | Call subroutine    |
| RET      | 0x40 | Return             |
| SYSCALL  | 0x41 | System call        |

### Stack Operations (0x43-0x55)

| Opcode | Hex  | Description            |
| ------ | ---- | ---------------------- |
| DEPTH  | 0x43 | Push stack depth       |
| DROP   | 0x45 | Remove top item        |
| DUP    | 0x4A | Duplicate top item     |
| SWAP   | 0x50 | Swap top two items     |
| ROT    | 0x51 | Rotate top three items |

### Arithmetic (0x9E-0xBB)

| Opcode      | Hex       | Description    |
| ----------- | --------- | -------------- |
| ADD         | 0x9E      | Addition       |
| SUB         | 0x9F      | Subtraction    |
| MUL         | 0xA0      | Multiplication |
| DIV         | 0xA1      | Division       |
| MOD         | 0xA2      | Modulo         |
| INC         | 0x9C      | Increment      |
| DEC         | 0x9D      | Decrement      |
| LT/LE/GT/GE | 0xB5-0xB8 | Comparisons    |

### Crypto System Calls

Neo N3 crypto operations are invoked through `SYSCALL` with canonical interop
hashes. The assembler keeps `SHA256`, `RIPEMD160`, `HASH160`, and `CHECKSIG` as
convenience aliases, but they emit `SYSCALL <interop-hash>` instead of private
opcodes.

| Alias     | Emitted Instruction               | Description            |
| --------- | --------------------------------- | ---------------------- |
| SHA256    | `SYSCALL System.Crypto.SHA256`    | SHA-256 hash           |
| RIPEMD160 | `SYSCALL System.Crypto.RIPEMD160` | RIPEMD-160 hash        |
| HASH160   | `SYSCALL System.Crypto.Hash160`   | SHA256 + RIPEMD160     |
| CHECKSIG  | `SYSCALL System.Crypto.CheckSig`  | Verify ECDSA signature |

`0xF1` is the canonical `THROWIFNOT` opcode. `0xF0`, `0xF2`, and `0xF3` are not
crypto opcodes in the canonical VM profile.

## Exit Codes

| Code | Meaning                                        |
| ---- | ---------------------------------------------- |
| 0    | Success                                        |
| 1    | Error (invalid input, execution failure, etc.) |

## Environment Variables

The CLI currently does not read environment variables for runtime flags.
Use explicit command-line options such as `--gas` / `-g`.

## See Also

- [Neo N3 VM Specification](https://docs.neo.org/docs/n3/reference/neo_vm.html)
- [Neo zkVM Architecture](./architecture.md)
