pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod reader;
pub mod assembler;
pub mod memory;
pub mod elf;

// use rustv::spec::{InstructionFormat, RV32I, Extension};
use assembler::Assembler;
use lexer::Lexer;
use tokenizer::Tokenizer;
use parser::Parser;
use memory::{Memory, BasicMemory};
use elf::write_elf;

fn main() {
    let code = "
            .text
            .globl main
        //this is gonna be great\n
        main:
            li   a0, 0

            lw   ra, -12(sp)
            lw   s0, +8(sp)
            addi x3, sp, 16 + 9
            ret
    ";
    let code1 = "
            .text
            .globl main
        //this is gonna be great\n
        main:
            // li   a0, 0
            // lw   ra, -12(sp)
            // lw   s0, +8(sp)
            addi x3, sp, 16 + 9
            ret
    ";
    let li = "
            li   t1, 3
    ";
    let addi = "
            addi   t3, t2, 8
    ";

    let mut t = syntax::gas::Tokenizer;
    let l = syntax::gas::Lexer;
    let p = syntax::gas::Parser;
    let s = syntax::gas::Assembler;
    let mut m = BasicMemory::new();
    let tokens = t.get_tokens(code1);
    let lexemes = l.parse(tokens);
    let stats = p.parse(lexemes);
    let words = s.to_words(stats);

    for w in &words {
        println!("{w:x}");
    }
    m.append_words(words);
    m.dump("test.txt").unwrap();

    write_elf("main.o", m.get_bytes()).unwrap();
}
