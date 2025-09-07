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

use crate::tokenizer::Tokenizer;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::assembler::Assembler;
use crate::emu::machine::Machine;

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

            .section .text
        _start:
            // write(stdout=1, msg, len)
            li a0, 1              // fd = 1 (stdout)
            la a1, msg            // buffer address
            li a2, 13             // length
            li a7, 64             // syscall: write
            ecall

            // exit(0)
            li a0, 0              // status
            li a7, 93             // syscall: exit
            ecall
    ";

    let mut tokenizer = syntax::gas::Tokenizer;
    let lexer = syntax::gas::Lexer;
    let parser = syntax::gas::Parser;
    let assembler = syntax::gas::Assembler;

    let tokens = tokenizer.get_tokens(code);
    println!("{:?}", &tokens);

    let lexemes = lexer.parse(tokens);
    println!("{:?}", &lexemes);

    let blocks = parser.parse(lexemes);
    println!("{:?}", &blocks);

    let output = assembler.to_words(blocks);
    println!("{:?}", output);

    // Export to ELF
    // let outputfile = "main.o";
    // let code = "
    //     li a7, 93
    //     li a0, 1000
    //     ecall
    // ";
    // utils::encode_to_elf(code, outputfile).unwrap();

    // Read ELF and execute the Machine
    // let inputfile = "main.o";
    // let mut m = utils::new_machine_from_elf_textsection(inputfile);
    // m.decode();
    // m.decode();
    // assert!(m.assert_reg(17u32, 93));
    // assert!(m.assert_reg(10u32, 1000));

    // Run with GDB support
    // let memsize = 1024*1024;
    // let port = 9999u16;
    // let riscv32_dbg = utils::wait_for_new_debugger_at_port(memsize, port);
    // riscv32_dbg.custom_gdb_event_loop_thread();
}
