	.globl _start

	.section .data
msg:
	.ascii "Hello world!\n"

	.section .text
_start:
	li a0, 1
	la a1, msg
	li a2, 13
	li a7, 64
	ecall

	li a0, 0
	li a7, 93
	ecall

