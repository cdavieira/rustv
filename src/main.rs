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
pub mod elfreader;

use crate::machine::Machine;

fn main() {
    let memsize = 1024*1024;
    let port = 9999u16;
    let riscv32_dbg = utils::wait_for_new_debugger_at_port(memsize, port);
    riscv32_dbg.custom_gdb_event_loop_thread();

    // let code = "
    //     li a7, 93
    //     li a0, 1000
    //     ecall
    // ";
    // let outputfile = "main2.o";
    // utils::encode_to_elf(code, outputfile);

    // let inputfile = "main2.o";
    // let mut m = utils::new_machine_from_elf_textsection(inputfile);
    // m.decode();
    // m.decode();
    // assert!(m.assert_reg(17u32, 93));
    // assert!(m.assert_reg(10u32, 1000));
}
