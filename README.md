# emu6

Very simple RISC-V emulator. Reads ELF file and execute in emulator.

## Usage

Basic usage with an interactive debug console:

```bash
emu6 <ELF File> -d
```

Currently the CPU configuration is generated from ELF files.

Use `emu6 --help` for further usage instructions.

## Features

RISC-V instruction set and features:

- [x] RV32I
- [x] RV64I
- [ ] Extension M
- [ ] Extension A
- [ ] Extension F
- [ ] Extension D
- [ ] Extension C
- [ ] Extension V
- [x] Zicsr
- [ ] Sv32
- [ ] Sv39
- [ ] PLIC
- [ ] CLINT
