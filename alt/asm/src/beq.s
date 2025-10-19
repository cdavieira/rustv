	.globl _start
	.section .text
_start:
	li t0, 100
	li t1, 100
	beq t0, t1, mylabel
	bne t1,t2, mylabel
	li t2, 200
mylabel:
	li a7, 93       # Linux syscall: exit
	li a0, 0        # return code 0
	ecall
