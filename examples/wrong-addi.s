li a7, 93     // Linux syscall: exit
addi  rb, 1, 16
li a0, 0      // return code 0
ecall
