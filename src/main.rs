pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod reader;
pub mod assembler;
pub mod memory;
pub mod elf;
pub mod misc;

use assembler::Assembler;
use lexer::Lexer;
use tokenizer::Tokenizer;
use parser::Parser;
use memory::{Memory, BasicMemory};
use elf::write_elf;

fn main() {
    //TODO: CRIAR NO PARSER A STRUCT DEFINIDA NO PASSO 2.2 DO DESIGN 2
    // let mut rv32i_cache = misc::ExtensionCache::<RV32I>::new();
    // let or2 = Rc::clone(&rv32i_cache.get_or_create(RV32I::OR, || Rc::new(RV32I::OR)));
    let code = "
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
    let mut m = BasicMemory::new();

    let tokens = t.get_tokens(code);
    let lexemes = l.parse(tokens);
    let stats = p.parse(lexemes);
    for w in &stats {
        println!("{:?}", w);
    }

    // let words = s.to_words(stats);
    //
    // for w in &words {
    //     println!("{w:x}");
    // }
    // m.append_words(words);
    // m.dump("test.txt").unwrap();
    //
    // write_elf("main.o", m.get_bytes()).unwrap();
}
