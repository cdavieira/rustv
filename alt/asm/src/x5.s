# I wrote this program but its not actually printing anything

# UPDATE: the problem is that 'la a1, myvar3' is getting translated into 'addi
# a1,gp,-2040' and gp doesn't have the right address (it's 0)
# Usually its the C runtime which sets gp to _start but since i'm writing
# assembly directly, # that doesn't happen (because gcc is not there for us)
	.globl _start
	.section .text
_start:
	# write(stdout=1, msg, len)
	li a0, 1
	la a1, myvar3
	li a2, 13
	li a7, 64
	ecall

	# exit(0)
	li a0, 0
	li a7, 93
	ecall

	.section .data
myvar1:
	.word 0x10
myvar2:
	.word 25
myvar3:
	.ascii "Hello world!\n"

