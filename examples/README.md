# Neo zkVM Examples

## Basic Arithmetic
```
; add.neoasm - Add two numbers
PUSH2
PUSH3
ADD
RET
```

## Factorial
```
; factorial.neoasm
PUSH5      ; n = 5
PUSH1      ; result = 1
; loop: result *= n; n--
```
