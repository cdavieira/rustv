mod spec;
mod tokenizer;
mod lexer;
mod parser;
mod syntax;
mod reader;

use lexer::Lexer;
use rustv::spec::{Instruction, RV32I, Extension};
use tokenizer::Tokenizer;
use parser::Parser;
use syntax::intel::{Statement};

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
            addi   t3, t2, t1
    ";

    let mut t = syntax::intel::Tokenizer;
    let l = syntax::intel::Lexer;
    let p = syntax::intel::Parser;

    let tokens = t.get_tokens(code);
    let lexemes = l.parse(tokens);
    let stats = p.parse(&lexemes);
    println!("{:?}", stats);

    let tokens = t.get_tokens(li);
    let lexemes = l.parse(tokens);
    let li = p.parse(&lexemes);

    let tokens = t.get_tokens(addi);
    let lexemes = l.parse(tokens);
    let addi = p.parse(&lexemes);

    //ori 5,5,0x800
    // let i: u32 = RV32I::ORI.get_bytes(5, 0, 5, 0x800, 0);
    // println!("ORI (bin): {i:b}");
    // println!("ORI (hex): {i:x}");
    //sw 5,0x3(6)
    // let i: u32 = RV32I::SW.get_bytes(6, 5, 0, 0, 3);
    // println!("SW (bin):  {i:b}");
    // println!("SW (hex):  {i:x}");
}
