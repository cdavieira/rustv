src := src/x5.s
obj := $(subst src,obj,$(src:.s=.o))
log := $(subst src,log,$(src:.s=.txt))
elf := /home/carlos/projs/riscv32/initramfs/home/$(notdir $(subst .o,,$(obj)))

AS := riscv32-unknown-linux-gnu-as
ASFLAGS := -g

LD := riscv32-unknown-linux-gnu-ld
LDFLAGS :=

HEXDUMP := hexdump

READELF := riscv32-unknown-linux-gnu-readelf

OBJDUMP := riscv32-unknown-linux-gnu-objdump

all:
	$(AS) $(ASFLAGS) $(src) -o $(obj)
	@echo "$(HEXDUMP) $(obj)" > $(log)
	$(HEXDUMP) $(obj)     >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "$(READELF) -a $(obj)" >> $(log)
	$(READELF) -a $(obj)  >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "$(OBJDUMP) -d $(obj)" >> $(log)
	$(OBJDUMP) -d $(obj)  >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	$(LD) $(LDFLAGS) $(obj) -o $(elf)

