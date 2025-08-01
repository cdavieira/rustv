	.text
	.globl	main
# I, B, S, U, R
# J
main:
	addi    sp, sp, 16
	sw      t0,3(t1)
	#beq     t1,t2,0x900
	beq     t1,t2,10
	lui     t3,25
	add     t3,t2,t1
	jal     t4,0x1c
	lw      ra, -12(sp)


	ret
