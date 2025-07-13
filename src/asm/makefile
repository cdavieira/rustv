all:
	riscv32-unknown-linux-gnu-as addi.s
	@echo "hexdump a.out" > info.txt
	hexdump a.out        >> info.txt
	@echo -e "\n\n\n"    >> info.txt
	@echo "riscv32-unknown-linux-gnu-readelf -a a.out" >> info.txt
	riscv32-unknown-linux-gnu-readelf -a a.out         >> info.txt
	@echo -e "\n\n\n"    >> info.txt
	@echo "riscv32-unknown-linux-gnu-objdump -d a.out" >> info.txt
	riscv32-unknown-linux-gnu-objdump -d a.out         >> info.txt
	@echo -e "\n\n\n"    >> info.txt
