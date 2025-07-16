	.text
	.globl main
	# this is gonna be great
main:
	li   a0, 0
	lw   ra, -12(sp)
	lw   s0, +8(sp)
	addi x3, sp, 16 + 9
	ret
