        .globl _start

// This program sorts an array of u32 (from the greatest to the lowest)

// In order to sort an array of length N:
// 1. Modify the variable 'myvector' in the data section
// 2. In '_start': set 'a2' to have the length of 'myvector'
// 3. Run the program :)

// Useful GDB commands to help with debugging:
// 'x/x 0x10074'
// 'info registers'
// 'info address myvector'
// 'si'
// 'c'

// Registers used:
// t0 = stores the address of the 'myvector' variable
// t1 = stores the index of the loop (referred to as 'i')
// t2 = stores the index of the inner loop (referred to as 'j')
// t3 = stores the element at 'myvector[i]'
// t4 = stores the element at 'myvector[j]'
// t5 = temporarily stores the element at 'myvector[i]' before element swap
// t6 = a pointer to either the address of 't3' or 't4' used to swap both elements
// a2 = stores the number of elements of the 'myvector' variable
// a3 = stores the index of the i'th element of 'myvector'
// a4 = stores the index of the j'th element of 'myvector'
// a5 = stores a2 - 1

// Estimate of the maximum number of steps executed by this program (worst case scenario)
// Math:
//  n = number of elements in 'myvector'
//  considering n > 1:
//  res = 5 + (n-1) * ( 6 + ( 4 + 14 * Summation(n-1) ) ) + 3
// Python:
//  f = lambda n: 5 + (n-1) * ( 6 + ( 4 + 14 * sum(range(0, n)) ) ) + 3
// Max number of steps required to sort an array with n elements:
//  f(1) = 10 steps (f only works for n > 1. This result comes from manual inspection)
//  f(2) = 32 steps
//  f(3) = 112 steps
//  f(4) = 290 steps
//  f(5) = 608 steps

	.section .text
_start:
        la t0, myvector
	li  a2, 3
	addi a5, a2, -1
loop:
	loop_init_list:
	li  t1, 0
	li  a3, 0

	loop_condition:
	beq t1, a5, loop_end

	loop_body:
	//inner_loop:
		inner_loop_init_list:
			mv   t2, t1
			addi t2, t2, 1            // t2 = t1 + 1
			mv  a4, a3
			addi  a4, a4, 4           // a4 = a3 + 4

		inner_loop_condition:
			beq t2, a2, inner_loop_end

		inner_loop_body:
			add t3, t0, a3  // t3 = t0 + a3 (myvector + 4*i)
			lw  t3, 0(t3)   // t3 = myvector[4*i]
			add t4, t0, a4    // t4 = t0 + a4 (myvector + 4*j)
			lw  t4, 0(t4)     // t4 = myvector[4*j]
			// if myvector[j] < myvector[i]
			//   continue;
			// else
			//   swap i'th element (t3) with j'th element (t4);
			blt t4, t3, inner_loop_increment
			mv t5, t3        // t5 = myvector[i]
			add t6, t0, a3   // t6 = t0 + a3 (myvector + 4*i)
			sw t4, 0(t6)     // *(myvector + 4*i) = t4
			add t6, t0, a4   // t6 = t0 + a4 (myvector + 4*j)
			sw t5, 0(t6)     // *(myvector + 4*j) = t5

		inner_loop_increment:
			addi t2, t2, 1
			addi a4, a4, 4
			beq  x0, x0, inner_loop_condition // we've found ourselves in an (inner) loop :')

		inner_loop_end:

	loop_increment:
        addi t1, t1, 1
	addi a3, a3, 4
	beq  x0, x0, loop_condition // we've found ourselves in a loop :')

	loop_end:

	// exit(0)
        li a7, 93
        li a0, 0
        ecall


        .section .data
// myvector: .word 1, 2
// myvector: .word 10, 3, 5
myvector: .word 3, 5, 10
// myvector: .word 10, 5, 3, 8, 20
