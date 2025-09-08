src := src/x7.s
obj := $(subst src,obj,$(src:.s=.o))
log := $(subst src,log,$(src:.s=.txt))
elf := /home/carlos/projs/riscv32/initramfs/home/$(notdir $(subst .o,,$(obj)))

AS := riscv32-unknown-linux-gnu-as
ASFLAGS := -g

LD := riscv32-unknown-linux-gnu-ld
LDFLAGS :=

HEXDUMP := hexdump
HEXDUMPFLAGS := -C

READELF := riscv32-unknown-linux-gnu-readelf
READELFFLAGS := -a

OBJDUMP := riscv32-unknown-linux-gnu-objdump
OBJDUMPFLAGS := -d

all:
	$(AS) $(ASFLAGS) $(src) -o $(obj)
	@echo "$(HEXDUMP) $(HEXDUMPFLAGS) $(obj)" > $(log)
	$(HEXDUMP) $(HEXDUMPFLAGS) $(obj) >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "$(READELF) $(READELFFLAGS) $(obj)" >> $(log)
	$(READELF) $(READELFFLAGS) $(obj) >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "$(OBJDUMP) $(OBJDUMPFLAGS) $(obj)" >> $(log)
	$(OBJDUMP) $(OBJDUMPFLAGS) $(obj) >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	$(LD) $(LDFLAGS) $(obj) -o $(elf)

