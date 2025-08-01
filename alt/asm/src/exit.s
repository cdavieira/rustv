.globl _start

# main:
    # ret
_start:
    li a7, 93       # Linux syscall: exit
    li a0, 0        # return code 0
    ecall           # make the syscall
