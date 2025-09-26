	.globl _start
	.section .data
var1: .word 0x4
	.section .text
_start:
	la t1, var1
	lw t2, 7(t1)
