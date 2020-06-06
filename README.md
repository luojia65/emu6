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

The target of this project is to support emulating heterogeneous CPUs and several
common SoC peripherals.
For example it's expected to support one Cortex-M and one RISC-V core sharing same
memory region and run at the same time.

Software features:

- [x] Load one ELF file
- [ ] Interactive debug shell
- [ ] DTB support
- [ ] Support multiple ELF files
- [ ] RISC-V ISA support
- [ ] Thumb-2 ISA support
- [ ] GDB server
- [ ] Cache model

RISC-V instruction set and features:

- [x] RV32I
- [x] RV64I
- [ ] RV128I
- [ ] Extension M
- [ ] Extension A
- [ ] Extension F
- [ ] Extension D
- [x] Extension C
- [ ] Extension V
- [x] Zicsr
- [ ] Zifencei
- [ ] User mode
- [ ] Supervisor mode
- [ ] Sv32
- [ ] Sv39
- [ ] Sv48
- [ ] PLIC
- [ ] CLINT
