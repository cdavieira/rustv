        .globl _start
	.section .text
_start:
        la sp, 64(stacktop)
        li a2, 1
        jal ra, myrecfunc
        li a7, 93
        li a0, 1000
        ecall
myrecfunc:
        // allocate new function frame and function pointer
        addi sp, sp, -16
        sw ra, 12(sp)    // 12(sp) - 16(sp) = ra
        sw fp, 8(sp)     // 8(sp) - 12(sp) = fp
        addi fp, sp, 16  // fp -> stack base
        // Do function work
        beq a2, 0, end
        addi a2, a2, -1
        jal ra, myrecfunc
end:
        // Popping function frame
        lw ra, 12(sp)
        lw fp, 8(sp)
        addi sp, sp, 16
        ret


        .section .data
stacktop:
	.skip 64

