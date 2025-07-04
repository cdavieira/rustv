mod spec;
mod tokenizer;
mod lexer;
mod parser;
mod syntax;
mod machine;
mod reader;
mod executor;
mod cpu;
mod memory;

use lexer::Lexer;
use executor::Executor;
use tokenizer::Tokenizer;
use parser::Parser;
use syntax::intel::{Statement::Instruction, Command, Opcode};

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
    // println!("{:?}", stats);

    let tokens = t.get_tokens(li);
    let lexemes = l.parse(tokens);
    let li = p.parse(&lexemes);

    let tokens = t.get_tokens(addi);
    let lexemes = l.parse(tokens);
    let addi = p.parse(&lexemes);

    let mut m = machine::BasicMachine::new();
    let e = executor::StatementExecutor;
    // let addi = stats.iter().find(|s| {
    //     if let Instruction { opcode, .. } = s {
    //         if let Command::OP(Opcode::RV32I(o)) = opcode {
    //             match o {
    //                 crate::spec::extensions::rv32i::Opcode::ADDI => {
    //                     true
    //                 },
    //                 _ => {
    //                     false
    //                 }
    //             }
    //
    //         }
    //         else {
    //             false
    //         }
    //     }
    //     else {
    //         false
    //     }
    // }).unwrap();

    m.info();
    e.execute(&mut m.cpu, addi.get(0).unwrap());
    e.execute(&mut m.cpu, li.get(0).unwrap());
    m.info();
}
