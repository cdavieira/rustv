    .section .data
msg:
    .asciz "Hello, World!\n"

    .section .text
    .globl _start
_start:
    # write(fd=1, buf=msg, count=14)
    li   a7, 64        # syscall number for write (Linux RISC-V)
    li   a0, 1         # file descriptor 1 (stdout)
    la   a1, msg       # address of string
    li   a2, 14        # length of string
    ecall              # invoke syscall

    # exit(0)
    li   a7, 93        # syscall number for exit (Linux RISC-V)
    li   a0, 0         # exit code 0
    ecall
