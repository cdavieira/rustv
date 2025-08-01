	.section .data
	.globl myvar
myvar:
	.word 32

	.section .text
	.globl _start
_start:
	la t0, myvar

	# exit(0)
	li   a7, 93           # exit syscall number
	li   a0, 0            # exit code
	ecall
