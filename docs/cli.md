# Neo zkVM CLI

## Commands

### run
Execute a script.
```bash
neo-zkvm run <hex_script>
neo-zkvm run 12139E40
```

### prove
Generate ZK proof.
```bash
neo-zkvm prove <hex_script>
```

### asm
Assemble source code.
```bash
neo-zkvm asm "PUSH2 PUSH3 ADD RET"
```

### disasm
Disassemble bytecode.
```bash
neo-zkvm disasm 12139E40
```
