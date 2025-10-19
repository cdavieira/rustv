	.globl _start
	.section .text
_start:
	// prepare stack
	la sp, 64(stacktop)
	jal ra, myfunc
	li a7, 93
	li a0, 1000
	ecall
	nop
	nop
	nop
myfunc:
	// Allocate new function frame
	addi sp, sp, -16
	sw ra, 12(sp) // 12(sp) - 16(sp) = ra
	sw fp, 8(sp)  // 8(sp) - 12(sp) = fp
	addi fp, sp, 16 // fp -> stack base

	// Function body goes here
	la a1, -4(fp)   // a1 = ra

	// Popping function frame
	lw ra, 12(sp)
	lw fp, 8(sp)
	addi sp, sp, 16
	ret

	.section .data
stacktop:
	.skip 64
