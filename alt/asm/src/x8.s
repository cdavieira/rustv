	.globl _start

	.section .data
var1: .word 0x4
var2: .word 0xa

	.section .text
_start:
la t1, var1
li t2, 100
sw t2, 4(t1)

