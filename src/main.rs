mod spec;
mod tokenizer;
mod lexer;
mod parser;
mod syntax;

use lexer::Lexer;
use crate::parser::Parser;

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

    let tokenizer = syntax::intel::Tokenizer;
    let lexer = syntax::intel::Lexer;
    let parser = syntax::intel::Parser;

    let tokens = tokenizer::get_tokens(&tokenizer, code);
    let lexemes = lexer.parse(tokens);
    let stats = parser.get_instructions(&lexemes);

    println!("{:?}", stats);
}
