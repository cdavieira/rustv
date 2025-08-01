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

use assembler::Assembler;
use lexer::Lexer;
use tokenizer::Tokenizer;
use parser::Parser;
use machine::{Machine, SimpleMachine};
use elf::{read_elf, write_elf};

fn main() {
    // let code = "
    //     li a7, 93
    //     li a0, 1000
    //     ecall
    // ";
    let code = "
            // .data
            // .word var 32

            .text
            .globl main
        //this is gonna be great\n
        main:
            li   a0, 0

            lw   ra, -12(sp)
            lw   s0, +8(sp)
            jal   s0, main
            addi x3, sp, 16 + 9
            ret
    ";
    let mut t = syntax::gas::Tokenizer;
    let l = syntax::gas::Lexer;
    let p = syntax::gas::Parser;
    let s = syntax::gas::Assembler;

    let tokens = t.get_tokens(code);
    let lexemes = l.parse(tokens);
    let stats = p.parse(lexemes);
    let words = s.to_words(stats);
    let mut m = SimpleMachine::new(&words);

    write_elf("main.o", m.bytes()).unwrap();
    if read_elf("a.out").is_ok() {
        m.decode();
        m.info();
    }
}
