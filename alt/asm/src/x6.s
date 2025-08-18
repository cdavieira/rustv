    .globl _start

    .section .data
msg:
    .ascii "Hello world!\n"   # 13 bytes including newline

    .section .text
_start:
    # write(stdout=1, msg, len)
    li a0, 1              # fd = 1 (stdout)
    la a1, msg            # buffer address
    li a2, 13             # length
    li a7, 64             # syscall: write
    ecall

    # exit(0)
    li a0, 0              # status
    li a7, 93             # syscall: exit
    ecall
