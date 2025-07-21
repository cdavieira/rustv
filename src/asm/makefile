src := x2.s
obj := $(src:.s=.o)
log := $(src:.s=.txt)

AS := riscv32-unknown-linux-gnu-as
HEXDUMP := hexdump
READELF := riscv32-unknown-linux-gnu-readelf
OBJDUMP := riscv32-unknown-linux-gnu-objdump

all:
	$(AS) $(src) -o $(obj)
	@echo "$(HEXDUMP) $(obj)" > $(log)
	$(HEXDUMP) $(obj)     >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "$(READELF) -a $(obj)" >> $(log)
	$(READELF) -a $(obj)  >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "$(OBJDUMP) -d $(obj)" >> $(log)
	$(OBJDUMP) -d $(obj)  >> $(log)
	@echo -e "\n\n\n"     >> $(log)
