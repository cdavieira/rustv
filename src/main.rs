pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod assembler;
pub mod memory;
pub mod elf;
pub mod cpu;
pub mod machine;
pub mod utils;
pub mod elfwriter;

use rustv::utils::uwords_to_bytes;

use crate::assembler::Assembler;
use crate::lexer::Lexer;
use crate::spec::AssemblySection;
use crate::tokenizer::Tokenizer;
use crate::parser::Parser;
use crate::machine::{Machine, SimpleMachine};
use crate::elf::{read_elf, write_elf, write_elf2};

fn main() {
    // let code = "
    //     li a7, 93
    //     li a0, 1000
    //     ecall
    // ";

    // let code = "
    //         // .data
    //         // .word var 32
    //
    //         .text
    //         .globl main
    //     //this is gonna be great\n
    //     main:
    //         li   a0, 0
    //
    //         lw   ra, -12(sp)
    //         lw   s0, +8(sp)
    //         jal   s0, main
    //         addi x3, sp, 16 + 9
    //         ret
    // ";
    // let mut t = syntax::gas::Tokenizer;
    // let l = syntax::gas::Lexer;
    // let p = syntax::gas::Parser;
    // let s = syntax::gas::Assembler;
    //
    // let tokens = t.get_tokens(code);
    // let lexemes = l.parse(tokens);
    // let stats = p.parse(lexemes);
    // let words = s.to_words(stats);
    // let mut m = SimpleMachine::new(&words);
    //
    // write_elf("main.o", m.bytes()).unwrap();
    // if read_elf("a.out").is_ok() {
    //     m.decode();
    //     m.info();
    // }

    // elf::info_elf("mult-sects");

    let mut t = syntax::gas::Tokenizer;
    let l = syntax::gas::Lexer;
    let p = syntax::gas::Parser;
    let s = syntax::gas::Assembler;

    let code2 = "
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
    let code = "
                .globl _start
                .section .text
        _start:
        //        li t0, 100
        //        li t1, 200
        //        blt t0, t1, mylabel

                .section .data
        myvar1:
                .word 0x10
        myvar2:
                .word 25

                .section .text
        mylabel:
                li a0, 0        // return code 0
                li a7, 93       // Linux syscall: exit
                ecall
    ";
    println!("{}", code2);

    let tokens = t.get_tokens(code2);
    // println!("{:?}", &tokens);

    let lexemes = l.parse(tokens);
    // println!("{:?}", &lexemes);

    let parser_output = p.parse(lexemes);

    let (metadata, mut symbol_table, section_table, sections) = parser_output.get_all();

    let sections: Vec<spec::AssemblyData> = sections
        .into_iter()
        .map(|section| {
            s.to_words(section)
        })
        .collect();


    let mut writer = elfwriter::ElfWriter::new();

    if symbol_table.contains_key("_start") {
        let (_, start_symbol_addr) = *symbol_table.get("_start").unwrap();
        writer.set_start_address(start_symbol_addr.try_into().unwrap());
        let _ = symbol_table.remove("_start").unwrap();
    }
    else {
        writer.set_start_address(0);
    }

    // TODO: handle what length actually means
    for (symb, (sect_name, rel_addr)) in symbol_table {
        let length = 0;
        writer.add_symbol(sect_name, rel_addr.try_into().unwrap(), &symb, length);
    }

    for section in sections {
        if section.data.len() > 0 {
            let name = section.name;
            let data = swap_words_endianness(uwords_to_bytes(&section.data));
            //REVIEW: THIS ALIGNMENT IS WRONG, I ONLY DID THIS TO TEST ONE THING
            match name {
                spec::AssemblySectionName::TEXT => {
                    writer.set_section_data(name, data, 4);
                },
                spec::AssemblySectionName::DATA => {
                    writer.set_section_data(name, data, 1);
                },
                _ => {}
            }
        }
    }

    writer.save("main2.o");
}

fn swap_words_endianness(v: Vec<u8>) -> Vec<u8> {
    let mut u: Vec<u8> = Vec::new();
    for b in v.chunks_exact(4) {
        u.push(b[3]);
        u.push(b[2]);
        u.push(b[1]);
        u.push(b[0]);
    }
    u
}
