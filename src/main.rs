pub mod tokenizer;
pub mod streamreader;
pub mod syntax;
pub mod lexer;
pub mod utils;
pub mod parser;
pub mod assembler;
pub mod emu {
    pub mod cpu;
    pub mod debugger;
    pub mod machine;
    pub mod memory;
}
pub mod lang {
    pub mod directive;
    pub mod ext;
    pub mod highassembly;
    pub mod lowassembly;
    pub mod pseudo;
}
pub mod obj {
    pub mod elfreader;
    pub mod elfwriter;
}

use env_logger::Env;
use rustv::utils::encode_to_word;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        println!("Usage");
        println!("  cargo run -- [ --build    | -b ] file.s");
        println!("  cargo run -- [ --debugger | -d ]");
        println!("  cargo run -- [ --decode-bin    ] 0x00001117");
        println!("  cargo run -- [ --decode-text   ] \"addi a2,a1,3\"");
        println!("  cargo run -- [ --elf      | -e ] file.s");
        return ;
    }

    if args.len() > 2 && matches!(args[1].as_str(), "--build" | "-b") {
        let srcfile = args[2].clone();
        let code = std::fs::read_to_string(srcfile).unwrap();

        eprintln!("Code representation builder");

        use crate::utils::build_code_repr;
        let _tools = build_code_repr(&code);
        // let data = tools.data_section_words();

        // use crate::utils::words_to_bytes_be;
        // let data = words_to_bytes_be(&data);

        // use crate::utils::print_bytes_hex;
        // print_bytes_hex(&data);

        return ;
    }

    if matches!(args[1].as_str(), "--debugger" | "-d") {
        let memsize = 1024*1024;
        let port    = 9999u16;

        eprintln!("Debugger mode");

        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

        let riscv32_dbg = utils::wait_for_new_debugger_at_port(memsize, port);

        riscv32_dbg.custom_gdb_event_loop_thread();
        // riscv32_dbg.default_gdb_event_loop_thread();

        return ;
    }

    if args.len() > 2 && matches!(args[1].as_str(), "--elf" | "-e") {
        let linker = "riscv32-unknown-linux-gnu-ld";
        let execfile = "main";
        let objectfile = "main.o";
        let srcfile = args[2].clone();

        eprintln!("Elf writter mode");

        let f = std::fs::read_to_string(srcfile).unwrap();

        utils::encode_to_elf(&f, objectfile).unwrap();

        let output = std::process::Command::new(linker)
            .arg(objectfile)
            .arg("-o")
            .arg(execfile)
            .output()
            .expect("Failed to link elf to executable");

        if output.status.success() {
            eprintln!("Sucess: code written to {}!", execfile)
        } else {
            eprintln!("Error: something went wrong :/")
        }
    }

    if args.len() > 2 && matches!(args[1].as_str(), "--decode-bin") {
        eprintln!("Binary instruction decode mode");
        // let word = 0x00000eef;
        // let word = 0x00001117;
        // let word = 0xff010113;
        use crate::lang::ext::InstructionFormat;

        let n = args[2].as_str().trim().replace("0x", "");

        let word = u32::from_str_radix(&n, 16).expect("Invalid instruction");

        let iformat = InstructionFormat::decode(word);

        println!("{:?}", iformat);

        return ;
    }

    if args.len() > 2 && matches!(args[1].as_str(), "--decode-text") {
        eprintln!("Text instruction decode mode");

        let code = args[2].as_str();

        let res = encode_to_word(code);

        println!("0x{:08x}", res);

        return ;
    }

    // Read ELF and execute the Machine (only text)
    // use crate::emu::machine::Machine as _;
    // let inputfile = "main.o";
    // let mut m = utils::new_machine_from_elf_textsection(inputfile);
    // m.decode();
    // m.decode();
    // assert!(m.assert_reg(17u32, 93));
    // assert!(m.assert_reg(10u32, 1000));

    // Read code and instantiate Machine from parser tools
    // use crate::utils::build_code_repr;
    // use crate::utils::new_machine_from_tools;
    // use crate::emu::machine::Machine as _;
    // let tools = build_code_repr(code);
    // let mut m = new_machine_from_tools(&tools);
    // m.decode();
    // m.decode();
    // println!("{:?}", m.words());

    // Read ELF and execute the Machine (text + data)
    // use crate::lang::highassembly::Register;
    // use crate::emu::machine::Machine as _;
    // let inputfile = "main";
    // let mut m = utils::new_machine_from_elf(inputfile);
    // m.decode();
    // assert!(m.assert_reg(Register::A0.id().into(), 1));
    // m.decode();
    // m.decode();
    // assert!(m.assert_reg(17u32, 93));
    // assert!(m.assert_reg(10u32, 1000));

    // Run instructions in memory
    // use crate::lang::lowassembly::DataEndianness;
    // use crate::emu::machine::SimpleMachine;
    // use crate::utils::encode_to_words;
    // use crate::emu::machine::Machine as _;
    // let words = encode_to_words(code);
    // let mut m = SimpleMachine::from_words(&words, DataEndianness::Be);
    // for _word in words {
    //     m.decode();
    // }
    // let r: Vec<_> = m
    //     .read_registers()
    //     .into_iter()
    //     .map(|reg| reg as i32)
    //     .collect();
    // println!("{:?}", r);
}
