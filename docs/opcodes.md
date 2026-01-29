# Neo zkVM Opcodes

This document provides a complete reference for all opcodes supported by Neo zkVM, following the Neo N3 specification.

## Opcode Categories

- [Constants](#constants)
- [Flow Control](#flow-control)
- [Stack Operations](#stack-operations)
- [Slot Operations](#slot-operations)
- [Splice Operations](#splice-operations)
- [Bitwise Operations](#bitwise-operations)
- [Arithmetic Operations](#arithmetic-operations)
- [Compound Types](#compound-types)
- [Type Operations](#type-operations)
- [Cryptographic Operations](#cryptographic-operations)

---

## Constants

Push constant values onto the evaluation stack.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| PUSHINT8 | 0x00 | 1 | Push 1-byte signed integer |
| PUSHINT16 | 0x01 | 1 | Push 2-byte signed integer |
| PUSHINT32 | 0x02 | 1 | Push 4-byte signed integer |
| PUSHINT64 | 0x03 | 1 | Push 8-byte signed integer |
| PUSHINT128 | 0x04 | 1 | Push 16-byte signed integer |
| PUSHINT256 | 0x05 | 1 | Push 32-byte signed integer |
| PUSHA | 0x0A | 1 | Push address (pointer) |
| PUSHNULL | 0x0B | 1 | Push null value |
| PUSHDATA1 | 0x0C | 1 | Push data with 1-byte length prefix |
| PUSHDATA2 | 0x0D | 1 | Push data with 2-byte length prefix |
| PUSHDATA4 | 0x0E | 1 | Push data with 4-byte length prefix |
| PUSHM1 | 0x0F | 1 | Push integer -1 |
| PUSH0 | 0x10 | 1 | Push integer 0 |
| PUSH1 | 0x11 | 1 | Push integer 1 |
| PUSH2 | 0x12 | 1 | Push integer 2 |
| PUSH3 | 0x13 | 1 | Push integer 3 |
| PUSH4 | 0x14 | 1 | Push integer 4 |
| PUSH5 | 0x15 | 1 | Push integer 5 |
| PUSH6 | 0x16 | 1 | Push integer 6 |
| PUSH7 | 0x17 | 1 | Push integer 7 |
| PUSH8 | 0x18 | 1 | Push integer 8 |
| PUSH9 | 0x19 | 1 | Push integer 9 |
| PUSH10 | 0x1A | 1 | Push integer 10 |
| PUSH11 | 0x1B | 1 | Push integer 11 |
| PUSH12 | 0x1C | 1 | Push integer 12 |
| PUSH13 | 0x1D | 1 | Push integer 13 |
| PUSH14 | 0x1E | 1 | Push integer 14 |
| PUSH15 | 0x1F | 1 | Push integer 15 |
| PUSH16 | 0x20 | 1 | Push integer 16 |

### Detailed Descriptions

#### PUSHINT8 (0x00)
Push a 1-byte signed integer onto the stack.
```
Operand: 1 byte (signed)
Stack: ... → ..., value
```

#### PUSHDATA1 (0x0C)
Push arbitrary data with a 1-byte length prefix (max 255 bytes).
```
Operand: 1 byte length + data
Stack: ... → ..., ByteString
```

#### PUSHNULL (0x0B)
Push a null reference onto the stack.
```
Stack: ... → ..., null
```

---

## Flow Control

Control program execution flow.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| NOP | 0x21 | 2 | No operation |
| JMP | 0x22 | 2 | Unconditional jump (1-byte offset) |
| JMP_L | 0x23 | 2 | Unconditional jump (4-byte offset) |
| JMPIF | 0x24 | 2 | Jump if true (1-byte offset) |
| JMPIF_L | 0x25 | 2 | Jump if true (4-byte offset) |
| JMPIFNOT | 0x26 | 2 | Jump if false (1-byte offset) |
| JMPIFNOT_L | 0x27 | 2 | Jump if false (4-byte offset) |
| JMPEQ | 0x28 | 2 | Jump if equal (1-byte offset) |
| JMPEQ_L | 0x29 | 2 | Jump if equal (4-byte offset) |
| JMPNE | 0x2A | 2 | Jump if not equal (1-byte offset) |
| JMPNE_L | 0x2B | 2 | Jump if not equal (4-byte offset) |
| JMPGT | 0x2C | 2 | Jump if greater than (1-byte offset) |
| JMPGT_L | 0x2D | 2 | Jump if greater than (4-byte offset) |
| JMPGE | 0x2E | 2 | Jump if greater or equal (1-byte offset) |
| JMPGE_L | 0x2F | 2 | Jump if greater or equal (4-byte offset) |
| JMPLT | 0x30 | 2 | Jump if less than (1-byte offset) |
| JMPLT_L | 0x31 | 2 | Jump if less than (4-byte offset) |
| JMPLE | 0x32 | 2 | Jump if less or equal (1-byte offset) |
| JMPLE_L | 0x33 | 2 | Jump if less or equal (4-byte offset) |
| CALL | 0x34 | 2 | Call subroutine (1-byte offset) |
| CALL_L | 0x35 | 2 | Call subroutine (4-byte offset) |
| CALLA | 0x36 | 2 | Call address from stack |
| CALLT | 0x37 | 2 | Call token |
| ABORT | 0x38 | 2 | Abort execution |
| ASSERT | 0x39 | 2 | Assert condition or abort |
| THROW | 0x3A | 2 | Throw exception |
| TRY | 0x3B | 2 | Begin try block (1-byte offsets) |
| TRY_L | 0x3C | 2 | Begin try block (4-byte offsets) |
| ENDTRY | 0x3D | 2 | End try block (1-byte offset) |
| ENDTRY_L | 0x3E | 2 | End try block (4-byte offset) |
| ENDFINALLY | 0x3F | 2 | End finally block |
| RET | 0x40 | 2 | Return from current context |
| SYSCALL | 0x41 | 16 | System call |

### Detailed Descriptions

#### JMP (0x22)
Unconditionally jump to the target address.
```
Operand: 1 byte signed offset
Stack: unchanged
```

#### JMPIF (0x24)
Pop the top value; if true, jump to target.
```
Operand: 1 byte signed offset
Stack: ..., condition → ...
```

#### CALL (0x34)
Call a subroutine at the specified offset.
```
Operand: 1 byte signed offset
Stack: pushes return address
```

#### RET (0x40)
Return from the current execution context.
```
Stack: unchanged (returns to caller)
```

---

## Stack Operations

Manipulate the evaluation stack.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| DEPTH | 0x43 | 2 | Push stack depth |
| DROP | 0x45 | 2 | Remove top item |
| NIP | 0x46 | 2 | Remove second-to-top item |
| XDROP | 0x48 | 2 | Remove item at index n |
| CLEAR | 0x49 | 2 | Clear entire stack |
| DUP | 0x4A | 2 | Duplicate top item |
| OVER | 0x4B | 2 | Copy second-to-top to top |
| PICK | 0x4D | 2 | Copy item at index n to top |
| TUCK | 0x4E | 2 | Insert top before second-to-top |
| SWAP | 0x50 | 2 | Swap top two items |
| ROT | 0x51 | 2 | Rotate top three items |
| ROLL | 0x52 | 2 | Move item at index n to top |
| REVERSE3 | 0x53 | 2 | Reverse top 3 items |
| REVERSE4 | 0x54 | 2 | Reverse top 4 items |
| REVERSEN | 0x55 | 2 | Reverse top n items |

### Detailed Descriptions

#### DUP (0x4A)
Duplicate the top stack item.
```
Stack: ..., a → ..., a, a
```

#### SWAP (0x50)
Swap the top two stack items.
```
Stack: ..., a, b → ..., b, a
```

#### ROT (0x51)
Rotate the top three items, bringing the third to the top.
```
Stack: ..., a, b, c → ..., b, c, a
```

#### PICK (0x4D)
Copy the item at index n (0-based from top) to the top.
```
Stack: ..., xn, ..., x0, n → ..., xn, ..., x0, xn
```

---

## Slot Operations

Access local variables, arguments, and static fields.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| INITSSLOT | 0x56 | 2 | Initialize static slot |
| INITSLOT | 0x57 | 2 | Initialize local and argument slots |
| LDSFLD0-5 | 0x58-0x5D | 2 | Load static field 0-5 |
| LDSFLD | 0x5E | 2 | Load static field n |
| STSFLD0-5 | 0x5F-0x64 | 2 | Store static field 0-5 |
| STSFLD | 0x65 | 2 | Store static field n |
| LDLOC0-6 | 0x66-0x6B | 2 | Load local variable 0-6 |
| LDLOC | 0x6C | 2 | Load local variable n |
| STLOC0-6 | 0x6D-0x72 | 2 | Store local variable 0-6 |
| STLOC | 0x73 | 2 | Store local variable n |
| LDARG0-6 | 0x74-0x79 | 2 | Load argument 0-6 |
| LDARG | 0x7A | 2 | Load argument n |
| STARG0-6 | 0x7B-0x80 | 2 | Store argument 0-6 |
| STARG | 0x81 | 2 | Store argument n |

### Detailed Descriptions

#### INITSLOT (0x57)
Initialize local and argument slots for the current context.
```
Operand: 2 bytes (local_count, arg_count)
Stack: ..., arg0, arg1, ..., argN → ...
```

#### LDLOC0 (0x66)
Load local variable 0 onto the stack.
```
Stack: ... → ..., local[0]
```

#### STLOC0 (0x6D)
Store top of stack into local variable 0.
```
Stack: ..., value → ...
```

---

## Splice Operations

String and buffer manipulation.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| NEWBUFFER | 0x88 | 2 | Create new buffer |
| MEMCPY | 0x89 | 2 | Copy memory |
| CAT | 0x8B | 2 | Concatenate two byte strings |
| SUBSTR | 0x8C | 2 | Extract substring |
| LEFT | 0x8D | 2 | Get left n bytes |
| RIGHT | 0x8E | 2 | Get right n bytes |

### Detailed Descriptions

#### CAT (0x8B)
Concatenate two byte strings.
```
Stack: ..., a, b → ..., a+b
```

#### SUBSTR (0x8C)
Extract a substring from a byte string.
```
Stack: ..., str, index, count → ..., substring
```

---

## Bitwise Operations

Bitwise and logical operations.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| INVERT | 0x90 | 8 | Bitwise NOT |
| AND | 0x91 | 8 | Bitwise AND |
| OR | 0x92 | 8 | Bitwise OR |
| XOR | 0x93 | 8 | Bitwise XOR |
| EQUAL | 0x97 | 8 | Check equality |
| NOTEQUAL | 0x98 | 8 | Check inequality |

### Detailed Descriptions

#### AND (0x91)
Perform bitwise AND on two integers.
```
Stack: ..., a, b → ..., a & b
```

#### OR (0x92)
Perform bitwise OR on two integers.
```
Stack: ..., a, b → ..., a | b
```

#### XOR (0x93)
Perform bitwise XOR on two integers.
```
Stack: ..., a, b → ..., a ^ b
```

---

## Arithmetic Operations

Mathematical operations on integers.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| SIGN | 0x99 | 8 | Get sign (-1, 0, or 1) |
| ABS | 0x9A | 8 | Absolute value |
| NEGATE | 0x9B | 8 | Negate value |
| INC | 0x9C | 8 | Increment by 1 |
| DEC | 0x9D | 8 | Decrement by 1 |
| ADD | 0x9E | 8 | Addition |
| SUB | 0x9F | 8 | Subtraction |
| MUL | 0xA0 | 8 | Multiplication |
| DIV | 0xA1 | 8 | Integer division |
| MOD | 0xA2 | 8 | Modulo |
| POW | 0xA3 | 8 | Power |
| SQRT | 0xA4 | 8 | Square root |
| MODMUL | 0xA5 | 8 | Modular multiplication |
| MODPOW | 0xA6 | 8 | Modular exponentiation |
| SHL | 0xA8 | 8 | Shift left |
| SHR | 0xA9 | 8 | Shift right |
| NOT | 0xAA | 8 | Logical NOT |
| BOOLAND | 0xAB | 8 | Logical AND |
| BOOLOR | 0xAC | 8 | Logical OR |
| NZ | 0xB1 | 8 | Not zero check |
| NUMEQUAL | 0xB3 | 8 | Numeric equality |
| NUMNOTEQUAL | 0xB4 | 8 | Numeric inequality |
| LT | 0xB5 | 8 | Less than |
| LE | 0xB6 | 8 | Less than or equal |
| GT | 0xB7 | 8 | Greater than |
| GE | 0xB8 | 8 | Greater than or equal |
| MIN | 0xB9 | 8 | Minimum of two values |
| MAX | 0xBA | 8 | Maximum of two values |
| WITHIN | 0xBB | 8 | Check if value is within range |

### Detailed Descriptions

#### ADD (0x9E)
Add two integers.
```
Stack: ..., a, b → ..., a + b
```

#### SUB (0x9F)
Subtract b from a.
```
Stack: ..., a, b → ..., a - b
```

#### MUL (0xA0)
Multiply two integers.
```
Stack: ..., a, b → ..., a * b
```

#### DIV (0xA1)
Integer division. Throws on division by zero.
```
Stack: ..., a, b → ..., a / b
```

#### MOD (0xA2)
Modulo operation. Throws on division by zero.
```
Stack: ..., a, b → ..., a % b
```

#### POW (0xA3)
Raise a to the power of b. Exponent must be non-negative.
```
Stack: ..., a, b → ..., a ^ b
```

#### WITHIN (0xBB)
Check if x is within range [a, b).
```
Stack: ..., x, a, b → ..., (a <= x && x < b)
```

---

## Compound Types

Operations on arrays, structs, and maps.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| PACKMAP | 0xBE | 2 | Pack items into map |
| PACKSTRUCT | 0xBF | 2 | Pack items into struct |
| PACK | 0xC0 | 2 | Pack items into array |
| UNPACK | 0xC1 | 2 | Unpack array to stack |
| NEWARRAY0 | 0xC2 | 2 | Create empty array |
| NEWARRAY | 0xC3 | 2 | Create array with n null elements |
| NEWARRAY_T | 0xC4 | 2 | Create typed array |
| NEWSTRUCT0 | 0xC5 | 2 | Create empty struct |
| NEWSTRUCT | 0xC6 | 2 | Create struct with n null elements |
| NEWMAP | 0xC8 | 2 | Create empty map |
| SIZE | 0xCA | 2 | Get size of compound type |
| HASKEY | 0xCB | 2 | Check if key exists |
| KEYS | 0xCC | 2 | Get all keys from map |
| VALUES | 0xCD | 2 | Get all values from map |
| PICKITEM | 0xCE | 2 | Get item by key/index |
| APPEND | 0xCF | 2 | Append item to array |
| SETITEM | 0xD0 | 2 | Set item by key/index |
| REVERSEITEMS | 0xD1 | 2 | Reverse array in place |
| REMOVE | 0xD2 | 2 | Remove item by key/index |
| CLEARITEMS | 0xD3 | 2 | Clear all items |
| POPITEM | 0xD4 | 2 | Pop last item from array |

### Detailed Descriptions

#### NEWARRAY (0xC3)
Create a new array with n null elements.
```
Stack: ..., n → ..., Array[null * n]
```

#### PICKITEM (0xCE)
Get an item from an array or map.
```
Stack: ..., container, key → ..., container[key]
```

#### SETITEM (0xD0)
Set an item in an array or map.
```
Stack: ..., container, key, value → ... (container modified)
```

#### APPEND (0xCF)
Append an item to an array.
```
Stack: ..., array, item → ... (array modified)
```

---

## Type Operations

Type checking and conversion.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| ISNULL | 0xD8 | 2 | Check if null |
| ISTYPE | 0xD9 | 2 | Check type |
| CONVERT | 0xDB | 2 | Convert to type |
| ABORTMSG | 0xE0 | 2 | Abort with message |
| ASSERTMSG | 0xE1 | 2 | Assert with message |

### Detailed Descriptions

#### ISNULL (0xD8)
Check if the top item is null.
```
Stack: ..., item → ..., (item == null)
```

#### CONVERT (0xDB)
Convert the top item to the specified type.
```
Operand: 1 byte (type code)
Stack: ..., item → ..., converted_item
```

---

## Cryptographic Operations

Hash functions and signature verification.

| Opcode | Hex | Gas | Description |
|--------|-----|-----|-------------|
| SHA256 | 0xF0 | 512 | SHA-256 hash |
| RIPEMD160 | 0xF1 | 512 | RIPEMD-160 hash |
| HASH160 | 0xF2 | 512 | SHA-256 + RIPEMD-160 |
| CHECKSIG | 0xF3 | 32768 | Verify ECDSA signature |

### Detailed Descriptions

#### SHA256 (0xF0)
Compute SHA-256 hash of the input.
```
Stack: ..., data → ..., sha256(data)
Output: 32 bytes
```

#### RIPEMD160 (0xF1)
Compute RIPEMD-160 hash of the input.
```
Stack: ..., data → ..., ripemd160(data)
Output: 20 bytes
```

#### HASH160 (0xF2)
Compute Hash160 (SHA-256 followed by RIPEMD-160).
```
Stack: ..., data → ..., ripemd160(sha256(data))
Output: 20 bytes
```

#### CHECKSIG (0xF3)
Verify an ECDSA secp256k1 signature.
```
Stack: ..., message, signature, pubkey → ..., valid
```

---

## Gas Consumption Summary

| Category | Gas Range | Examples |
|----------|-----------|----------|
| **Low** | 1-2 | PUSH*, NOP, JMP, RET, DUP, DROP |
| **Medium** | 8 | ADD, SUB, MUL, AND, OR, LT, GT |
| **High** | 16 | SYSCALL |
| **Very High** | 512 | SHA256, RIPEMD160, HASH160 |
| **Extreme** | 32768 | CHECKSIG |

### Gas Cost Table

```
┌─────────────────────────────────────────────────────────────┐
│                    GAS COST BREAKDOWN                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1 gas:    PUSH0-16, PUSHNULL, PUSHM1, PUSHDATA*            │
│  2 gas:    NOP, JMP*, CALL, RET, DUP, DROP, SWAP, ROT       │
│            INITSLOT, LDLOC*, STLOC*, LDARG*, STARG*         │
│            NEWARRAY*, NEWMAP, SIZE, PICKITEM, SETITEM       │
│  8 gas:    ADD, SUB, MUL, DIV, MOD, POW, SQRT               │
│            AND, OR, XOR, INVERT, SHL, SHR                   │
│            LT, LE, GT, GE, EQUAL, NUMEQUAL                  │
│            NOT, BOOLAND, BOOLOR, NZ, WITHIN                 │
│  16 gas:   SYSCALL                                          │
│  512 gas:  SHA256, RIPEMD160, HASH160                       │
│  32768 gas: CHECKSIG                                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Opcode Quick Reference

### By Hex Value

| Range | Category |
|-------|----------|
| 0x00-0x05 | PUSHINT (8/16/32/64/128/256) |
| 0x0A-0x20 | Constants (PUSHA, PUSHNULL, PUSHDATA, PUSH0-16) |
| 0x21-0x41 | Flow Control |
| 0x43-0x55 | Stack Operations |
| 0x56-0x81 | Slot Operations |
| 0x88-0x8E | Splice Operations |
| 0x90-0x98 | Bitwise Operations |
| 0x99-0xBB | Arithmetic Operations |
| 0xBE-0xD4 | Compound Types |
| 0xD8-0xE1 | Type Operations |
| 0xF0-0xF3 | Cryptographic Operations |
