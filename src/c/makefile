CC := riscv32-unknown-linux-gnu-gcc
# CC := clang
CFLAGS :=

ifeq ($(CC),clang)
CFLAGS += --target=riscv32
CFLAGS += -march=rv32i
CFLAGS += -mabi=ilp32
CFLAGS += -S
else
CFLAGS += -S
CFLAGS += -fno-unwind-tables -fno-asynchronous-unwind-tables
CFLAGS += -fno-dwarf2-cfi-asm
# CFLAGS += -O2
# CFLAGS += -nostdlib
# CFLAGS += -ffreestanding 
# CFLAGS += -fno-stack-protector 
endif

all:
	$(CC) $(CFLAGS) src/main.c -o main.s

clean:
	rm main.s
