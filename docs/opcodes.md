# Neo zkVM Opcodes

## Constants
| Opcode | Hex | Description |
|--------|-----|-------------|
| PUSH0-16 | 0x10-0x20 | Push integer 0-16 |
| PUSHM1 | 0x0F | Push -1 |
| PUSHNULL | 0x0B | Push null |
| PUSHDATA1 | 0x0C | Push data (1-byte len) |

## Arithmetic
| Opcode | Hex | Description |
|--------|-----|-------------|
| ADD | 0x9E | a + b |
| SUB | 0x9F | a - b |
| MUL | 0xA0 | a * b |
| DIV | 0xA1 | a / b |
| MOD | 0xA2 | a % b |
| POW | 0xA3 | a ^ b |
| INC | 0x9C | a + 1 |
| DEC | 0x9D | a - 1 |

## Stack
| Opcode | Hex | Description |
|--------|-----|-------------|
| DUP | 0x4A | Duplicate top |
| DROP | 0x45 | Remove top |
| SWAP | 0x50 | Swap top two |
| CLEAR | 0x49 | Clear stack |

## Control
| Opcode | Hex | Description |
|--------|-----|-------------|
| JMP | 0x22 | Jump |
| JMPIF | 0x24 | Jump if true |
| RET | 0x40 | Return |
