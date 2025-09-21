# rustv
This project aims to implement a RISC-V 32 bits emulator that can run programs
which use a subset of the RV32I instructions

The emulation works by:
1. Reading a single ELF file
2. Loading the data to an emulated machine, which has a CPU and Memory
3. Perform a fetch-decode cycle, where each instruction is decoded according to
   its binary format and then executed

This project can also assemble a single assembly file into an ELF object file.
This allows the program to assemble an assembly file and then execute the
resulting ELF file in the emulated machine.

The emulation can also be connected to GDB, in order to debug the program
during its execution.


## How to run
```bash
# compile
cargo build --release

# compile and run
cargo run

# run tests
cargo tests
```


## About ...

### The assembly-to-elf procedure
Riscv assembly code is turned into a valid ELF file through the following
steps:
1. The assembly code is turned into a stream of indivisible raw tokens
2. The stream of raw tokens are classified according to the usual assembly
   syntax (numbers, sections, symbols, opcodes, ...)
3. The stream of (now classified) tokens is reorganized in sections and
   instructions
4. All instructions are then resolved and translated to their binary format and
   exported to the ELF format
> this is done with the help of the `object` crate
5. The resulting object file is then linked using the `ld` linker (compiled to
   target riscv32 machines) in order to produce a runnable ELF executable

In the code, steps 1, 2, 3 and 4 are bound to traits, which have to be
implemented by any entity which takes charge of them.

| Step | Trait to be implemented |
| :--: |  :---------------:  |
|   1  | `Tokenizer`         |
|   2  | `Lexer`             |
|   3  | `Parser`            |
|   4  | `Assembler`         |

### The ISA Implementation
RISC-V has many extensions, all of which define a set of instructions to be supported

This project implements support for the core instructions of RISC-V (RV32I):
* LUI, AUIPC, ADDI, ANDI, ORI, XORI, ADD, SUB, AND, OR, XOR, SLL, SRL, SRA,
FENCE, SLTI, SLTIU, SLLI, SRLI, SRAI, SLT, SLTU, LW, LH, LHU, LB, LBU, SW, SH,
SB, JAL, JALR, BEQ, BNE, BLT, BLTU, BGE, BGEU, ECALL

All instructions from the RV32I extension can be used to produce valid ELF
object files from riscv32 assembly code and be executed in the emulated machine.

New extensions can be also supported through the implementation of the
`Extension` trait found in `ext.rs`

Syscalls are planned to be supported soon

### Pseudo instructions
In the code, a pseudo instruction is simply any type which implements the
`Pseudo` trait, which declares that all pseudo instructions must be translable
into a sequence of real instructions.

More on this can be found in `pseudo.rs`

### The Fetch/Decode cycle
The emulated machine works by mimicking the fetch-decode cycle performed in
real machines, where one instruction at a time is fetched from memory and then
executed.

### The Memory model
The memory model used by the machine consists of a flat memory of 1 MiB, whose
endianness can be either Little endian or Big endian. There's no byte-alignment
enforced by the memory itself.

### The CPU
The CPU model has 32 general purpose registers available and a PC special
register, all of which can be read and written

### Correctness
All tests can be found in `lib.rs` and can be run with `cargo test` or `cargo
test test_name`

### How Rust helps
Rust helped with the test-oriented design, where each entity/component can be
tested separately using the builtin support of cargo for tests

Language-wise, pattern matching was heavily used for instruction decoding,
tokenization and more. Besides that, traits were used to enforce and formalize
the job of each entity in code. In this regard, traits act as contracts, which
have to be met by any entity in code which implements them. This allowed
implementors to strictly follow the standards that were conceptualized, which
also eased the creation of tests.




## How this project is being tested/validated
All ELF files created are tested in QEMU (using `qemu-system-riscv32` to build
emulated machines), which runs the latest version of the linux kernel compiled
for riscv32. This emulated machine uses busybox as to provide a set of core gnu
utilities. This allows testing all executable files in a consolidated emulation
environment.

Besides that, the [RISCV-32
toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain) was compiled in
other to use tools such as `as`, `ld` and `gdb` to validate the correctness of
the resulting ELF files and if they worked as intended with real gnu tools.




## How to connect the emulated machine to gdb
```bash
# run the program with debugger support
cargo run

# in another terminal, run gdb
riscv32-unknown-linux-gnu-gdb main

# inside of gdb, connect to the remote target (which listens at port 9999 by
# default)
gdb> target remote :9999
# load the program to the virtual memory
gdb> load
# add breakpoints
gdb> b _start
# step/run the program execution
gdb> si
```



## Future work
* Create a graphical interface which allows writing/editing assembly code in
realtime, exporting to ELF and running the emulated environment
* Implement Pipelining



## TODO
* [ ] Reorganize README sections
* [ ] Talk about the problems/limitations (misalignment, out-of-range memory accesses, ...)
* [ ] Talk about what is missing assembly-wise (functions, traps, syscalls)
* [ ] Explain the current test suite (Tokenizer, Machine, CPU, Memory, ELF export, ...)
* [ ] Create diagrams/images explaining the Designs (assembly-to-elf, how the
gdbstub works, how the fetch-decode cycle works)




## references
* [(Official) The RISC-V Instruction Set Manual (Volumes I) - riscv org](https://lf-riscv.atlassian.net/wiki/spaces/HOME/pages/16154769/RISC-V+Technical+Specifications)
> document version: 20250508
* [RISC-V ISA Pages - riscv isadoc](https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html)
* [(Official) RISC-V Assembly Programmer's Manual - Github](https://github.com/riscv-non-isa/riscv-asm-manual)
* [Linux syscalls - Github](https://github.com/torvalds/linux/blob/master/arch/riscv/include/uapi/asm/unistd.h)
* [Syscalls for riscv32 - syscalls crate](https://docs.rs/syscalls/latest/syscalls/riscv32/enum.Sysno.html)
* [Errno values](https://gist.github.com/greggyNapalm/2413028)
