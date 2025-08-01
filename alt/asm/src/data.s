	.section .data

	.globl my_value
my_value:
	.word 1234

	.globl my_array
my_array:
	.word 1, 2, 3, 4, 5

	.section .bss
	.globl my_buffer
my_buffer:
	.skip 64


	.section .text
	.globl _start
_start:
	# Load address of my_value into t0
	la   t0, my_value

	# Load the value from memory into t1
	lw   t1, 0(t0)

	# Load address of my_array into t2
	la   t2, my_array

	# Load the third element (index 2) of my_array
	lw   t3, 8(t2)        # offset = 2 * 4 bytes

	# Load address of my_buffer into t4
	la   t4, my_buffer

	# Store t3 (array element) into first position of my_buffer
	sw   t3, 0(t4)

	# Exit program (Linux syscall)
	li   a7, 93           # exit syscall number
	li   a0, 0            # exit code
	ecall
