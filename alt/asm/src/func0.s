	.globl _start
	.section .text
_start:
	li t1, 3
	jal ra, myfunc
	li a7, 93
	li a0, 1000
	ecall
myfunc:
	add a0, a0, a1
	ret
