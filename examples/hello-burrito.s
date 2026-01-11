	.globl _start

	.section .data
msg:   .ascii "Hello world!\n" // 13 bytes including newline
msg2:  .ascii "Burrito!\n"     // 9  bytes including newline
myvar: .word  32

	.section .text
_start:
	// write(stdout=1, msg, len)
	li a0, 1              // fd = 1 (stdout)
	la a1, msg            // buffer address
	li a2, 13             // length
	li a7, 64             // syscall: write
	ecall
	la a1, msg            // buffer address
	la a1, msg            // buffer address

	// write(stdout=1, msg, len)
	write2:
	li a0, 1              // fd = 1 (stdout)
	xor a1,a1,a1
	la a1, msg2           // buffer address
	li a2, 9              // length
	li a7, 64             // syscall: write
	ecall
sub_op:
	sub a7,a2,t2
xor_op:
	xor a1,a1,a1
exit:
	// exit(0)
	li a0, 0              // status
	li a7, 93             // syscall: exit
	ecall
