The Linux calling convention for RISC-V 32-bit (RV32) follows the standard
RISC-V ABI (Application Binary Interface) for function calls and system calls.
Key aspects include:

1. Register Usage for Function Calls:
Arguments: Function arguments are primarily passed in registers a0 through a7.

Return Values: Return values are typically placed in registers a0 and a1.

Return Address: The ra register (x1) holds the return address, set by the jal
(Jump and Link) instruction.

Stack Pointer: The sp register (x2) points to the top of the stack. For RV32,
the stack pointer should be aligned to a 32-bit boundary. 

Callee-Saved Registers: Registers s0 through s11 (x8, x9, x18-x27) are
callee-saved, meaning the called function must preserve their values and
restore them before returning.

Caller-Saved/Temporary Registers: Registers t0 through t6 (x5-x7, x28-x31) are
caller-saved/temporary, meaning the calling function is responsible for saving
their values if they need to be preserved across a function call.




2. System Calls (ecall):
System Call Number: The system call number (e.g., 64 for write, 93 for exit) is
placed in register a7.

Arguments: System call arguments are passed in registers a0 through a6.

Execution: The ecall instruction is executed to invoke the kernel.

Return Value: The return value from the system call is placed in a0.




3. ABI Variations:
Integer ABIs: ilp32, ilp32e (for RV32E architecture), ilp32f (with
single-precision float support), and ilp32d (with double-precision float
support) are common ABIs for RV32.

Floating-Point Registers: If floating-point extensions are used, fa0 through
fa7 are used for passing floating-point arguments, and fa0/fa1 for return
values.

Note: Specific details may vary slightly depending on the exact RISC-V ISA
extensions and the compiler/toolchain used (e.g., GCC options like -mabi).
However, the general principles of register usage for arguments, return values,
and system calls remain consistent with the RISC-V calling convention
specification.
