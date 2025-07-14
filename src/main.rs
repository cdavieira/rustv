pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod reader;
pub mod assembler;

use assembler::Assembler;
use lexer::Lexer;
use rustv::spec::{Instruction, RV32I, Extension};
use tokenizer::Tokenizer;
use parser::Parser;

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
    let li = "
            li   t1, 3
    ";
    let addi = "
            addi   t3, t2, 8
    ";

    let mut t = syntax::intel::Tokenizer;
    let l = syntax::intel::Lexer;
    let p = syntax::intel::Parser;
    let s = syntax::intel::Assembler;

    let tokens = t.get_tokens(code);
    let lexemes = l.parse(tokens);
    // let stats = p.parse(&lexemes);

    let tokens = t.get_tokens(li);
    let lexemes = l.parse(tokens);
    // let li = p.parse(&lexemes);

    let tokens = t.get_tokens(addi);
    let lexemes = l.parse(tokens);
    let addi = p.parse(&lexemes);
    println!("{:?}", addi);

    let bits = s.to_words(&addi);
    println!("{:?}", bits);

    // let m: Vec<&Statement> = addi.iter().map(|e| e).collect();
    // let b = s.to_words(m);
    // for bits in b {
    //     println!("{bits:b}");
    //     println!("{bits:x}");
    // }

    // //ori 5,5,0x800
    // let i: u32 = RV32I::ORI.get_instruction(5, 0, 5, 0x800).get_bytes();
    // println!("ORI (bin): {i:b}");
    // println!("ORI (hex): {i:x}");
    // //sw 5,0x3(6)
    // let i: u32 = RV32I::SW.get_instruction(6, 5, 0, 0x3).get_bytes();
    // println!("SW (bin):  {i:b}");
    // println!("SW (hex):  {i:x}");
}
