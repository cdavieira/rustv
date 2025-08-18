	.globl _start
	.section .text
_start:
	li t0, 100
	li t1, 200
	blt t0, t1, mylabel
	.section .data
myvar1:
	.word 0x10
	.section .text
mylabel:
	li a0, 0        # return code 0
	li a7, 93       # Linux syscall: exit
	ecall
