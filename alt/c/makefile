# CC := riscv32-unknown-linux-gnu-gcc
CC := clang
CFLAGS :=

ifeq ($(CC),clang)
CFLAGS += --target=riscv32
CFLAGS += -march=rv32i
CFLAGS += -mabi=ilp32
# CFLAGS += -S
else
CFLAGS += -fno-unwind-tables -fno-asynchronous-unwind-tables
CFLAGS += -fno-dwarf2-cfi-asm
# CFLAGS += -S
# CFLAGS += -O2
# CFLAGS += -nostdlib
# CFLAGS += -ffreestanding 
# CFLAGS += -fno-stack-protector 
endif

all:
	$(CC) $(CFLAGS) main.c -S -o main.s
	$(CC) $(CFLAGS) main.c -o main

clean:
	rm main.s main
