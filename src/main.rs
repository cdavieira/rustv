pub mod tokenizer;
pub mod streamreader;
pub mod syntax;
pub mod lexer;
pub mod utils;
pub mod parser;
pub mod assembler;
pub mod emu {
    pub mod cpu;
    pub mod debugger;
    pub mod machine;
    pub mod memory;
}
pub mod lang {
    pub mod directive;
    pub mod ext;
    pub mod highassembly;
    pub mod lowassembly;
    pub mod pseudo;
}
pub mod obj {
    pub mod elfreader;
    pub mod elfwriter;
}


fn main() {
    // let code = "
    //     li a7, 93
    //     li a0, 1000
    //     ecall
    // ";

    // let code = "
    //             .globl _start
    //             .section .text
    //     _start:
    //     //        li t0, 100
    //     //        li t1, 200
    //     //        blt t0, t1, mylabel
    //
    //             .section .data
    //     myvar1:
    //             .word 0x10
    //     myvar2:
    //             .word 25
    //
    //             .section .text
    //     mylabel:
    //             li a0, 0        // return code 0
    //             li a7, 93       // Linux syscall: exit
    //             ecall
    // ";

    let code = "
            .globl _start

            .section .data
        msg:
            .ascii \"Hello world!\n\"   // 13 bytes including newline
        msg2:
            .ascii \"Burrito!\n\"   // 9 bytes including newline
        myvar:
            .word 32

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
    ";

    // let code = "
    //         .globl _start
    //         .section .text
    // _start:
    //         li t1, 3
    //         jal ra, myfunc
    //         li a7, 93
    //         li a0, 1000
    //         ecall
    // myfunc:
    //         add a0, a0, a1
    //         ret
    //         .section .data
    //         .skip 20
    //         .word 32
    // ";

    // let code = "
    //     .section .data
    //     var1: .word 0x4
    //     .section .text
    //         la t1, var1
    //         lw t2, t1
    // ";
    // let code = "
    //     li a2, 4
    //     jalr ra, a2, 8
    // ";



    // See how instruction decoding evals
    // use crate::lang::ext::InstructionFormat;
    // let word = 0x00000eef;
    // let iformat = InstructionFormat::decode(word);
    // println!("{:?}", iformat);

    // Build code representation
    // use crate::utils::build_code_repr;
    // use crate::utils::print_bytes_hex;
    // use crate::utils::words_to_bytes_be;
    // let tools = build_code_repr(code);
    // let data = tools.data_section_words();
    // let data = words_to_bytes_be(&data);
    // print_bytes_hex(&data);

    // Export to ELF
    let outputfile = "main.o";
    utils::encode_to_elf(code, outputfile).unwrap();

    // Read ELF and execute the Machine (only text)
    // use crate::emu::machine::Machine as _;
    // let inputfile = "main.o";
    // let mut m = utils::new_machine_from_elf_textsection(inputfile);
    // m.decode();
    // m.decode();
    // assert!(m.assert_reg(17u32, 93));
    // assert!(m.assert_reg(10u32, 1000));

    // Read code and instantiate Machine from parser tools
    // use crate::utils::build_code_repr;
    // use crate::utils::new_machine_from_tools;
    // use crate::emu::machine::Machine as _;
    // let tools = build_code_repr(code);
    // let m = new_machine_from_tools(&tools);
    // println!("{:?}", m.words());
    // println!("{:?}", m.read_registers());

    // Read ELF and execute the Machine (text + data)
    // use crate::lang::highassembly::Register;
    // use crate::emu::machine::Machine as _;
    // let inputfile = "main";
    // let mut m = utils::new_machine_from_elf(inputfile);
    // m.decode();
    // assert!(m.assert_reg(Register::A0.id().into(), 1));
    // m.decode();
    // m.decode();
    // assert!(m.assert_reg(17u32, 93));
    // assert!(m.assert_reg(10u32, 1000));

    // Run with GDB support
    // let memsize = 1024*1024;
    // let port = 9999u16;
    // let riscv32_dbg = utils::wait_for_new_debugger_at_port(memsize, port);
    // riscv32_dbg.custom_gdb_event_loop_thread();
    // riscv32_dbg.default_gdb_event_loop_thread();

    // Run instructions in memory
    // use crate::lang::lowassembly::DataEndianness;
    // use crate::emu::machine::SimpleMachine;
    // use crate::utils::encode_to_words;
    // use crate::emu::machine::Machine as _;
    // let words = encode_to_words(code);
    // let mut m = SimpleMachine::from_words(&words, DataEndianness::Be);
    // for _word in words {
    //     m.decode();
    // }
    // let r: Vec<_> = m
    //     .read_registers()
    //     .into_iter()
    //     .map(|reg| reg as i32)
    //     .collect();
    // println!("{:?}", r);
}
