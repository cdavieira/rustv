pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod assembler;
pub mod memory;
pub mod cpu;
pub mod machine;
pub mod utils;
pub mod elfwriter;
pub mod debugger;

use gdbstub::stub::state_machine::GdbStubStateMachine;

use crate::assembler::Assembler;
use crate::lexer::Lexer;
use crate::spec::AssemblySection;
use crate::tokenizer::Tokenizer;
use crate::parser::Parser;
use crate::machine::SimpleMachine;
use crate::utils::encode_to_elf;

fn main() {
    // let memsize = 1024*1024;
    // let port = 9999u16;
    // if let Ok(riscv32_dbg) = debugger::SimpleGdbStub::<SimpleMachine>::new(memsize, port) {
    //     riscv32_dbg.custom_gdb_event_loop_thread();
    // }
    // else {
    //     println!("Failed when instantiating riscv32 debugger");
    // }

    let code = "
        li a7, 93
        li a0, 1000
        ecall
    ";
    let outputfile = "main2.o";
    encode_to_elf(code, outputfile);
}
