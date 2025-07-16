src := x1.s
obj := $(src:.s=.o)
log := $(src:.s=.txt)
all:
	riscv32-unknown-linux-gnu-as $(src) -o $(obj)
	@echo "hexdump $(obj)" > $(log)
	hexdump $(obj)        >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "riscv32-unknown-linux-gnu-readelf -a $(obj)" >> $(log)
	riscv32-unknown-linux-gnu-readelf -a $(obj)         >> $(log)
	@echo -e "\n\n\n"     >> $(log)
	@echo "riscv32-unknown-linux-gnu-objdump -d $(obj)" >> $(log)
	riscv32-unknown-linux-gnu-objdump -d $(obj)         >> $(log)
	@echo -e "\n\n\n"     >> $(log)
