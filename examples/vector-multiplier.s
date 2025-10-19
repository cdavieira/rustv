	.globl _start
	.section .text
_start:
	la t0, myvector // address of 'myvector' variable
	li t2, 3        // size of 'myvector'
	li t3, -12      // multiplier
	li t4, 4        // size of each element
loop:
	loop_init_list:
	li  t1, 0

	loop_condition:
	beq t1, t2, loop_end

	loop_body:
	mul  t5, t1, t4 // idx to i'th element (4*i)
	add  t5, t0, t5 // address to the i'th element (myvector + 4*i)
	lw   t6, 0(t5)  // value of the i'th element (t6 = myvector[4*i])
	mul  t6, t3, t6 // multiplied value (t6 = factor * myvector[4*i])
	sw   t6, 0(t5)  // storing multiplied value  (*(myvector + 4*i) = t6)

	loop_increment:
	addi t1, t1, 1
	beq  x0, x0, loop_condition // we've found ourselves in a loop :')

	loop_end:

	li a7, 93
	li a0, 0
	ecall

	.section .data
myvector: .word 3, 5, 10
