	.section .data
var1: .word 0x4

	.section .text
	la t1, var1
	lw t2, 0(t1)
