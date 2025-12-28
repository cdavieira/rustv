CC := riscv32-unknown-linux-gnu-gcc
# CC := clang
CFLAGS :=

ifeq ($(CC),clang)
CFLAGS += --target=riscv32
CFLAGS += -march=rv32i
CFLAGS += -mabi=ilp32
else
CFLAGS += -fno-unwind-tables -fno-asynchronous-unwind-tables
CFLAGS += -fno-dwarf2-cfi-asm
CFLAGS += -march=rv32id
# CFLAGS += -O2
# CFLAGS += -nostdlib
# CFLAGS += -ffreestanding 
# CFLAGS += -fno-stack-protector 
endif

src := $(wildcard src/*.c)
obj := $(subst src,obj,$(src:.c=))

all: | objdir

objdir:
	mkdir -p obj

all: $(obj)

obj/%: src/%.c
	$(CC) $(CFLAGS) $< -S -o $@.s
	$(CC) $(CFLAGS) $< -o $@

clean:
	rm -rf obj
