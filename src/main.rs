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

fn main() {
    let arg_buffer: Vec<String> = std::env::args().collect();
    let args: Vec<&str> =  arg_buffer.iter().map(|arg| arg.as_str()).collect();
    let arglen = args.len();

    let show_usage    = arglen > 1 && matches!(args[1], "--help"     | "-h") || arglen == 1;
    let start_stub    = arglen > 1 && matches!(args[1], "--debugger" | "-d");
    let build_code    = arglen > 2 && matches!(args[1], "--build"    | "-b");
    let write_elf     = arglen > 2 && matches!(args[1], "--elf"      | "-e");
    let decode_binary = arglen > 2 && matches!(args[1], "--decode-bin"     );
    let decode_text   = arglen > 2 && matches!(args[1], "--decode-text"    );

    if show_usage {
        usage();
        return;
    }

    if build_code {
        let srcfile = args[2];
        let code = std::fs::read_to_string(srcfile).unwrap();

        use crate::utils::build_code_repr;
        let _tools = build_code_repr(&code);
        // let data = tools.data_section_words();

        // use crate::utils::words_to_bytes_be;
        // let data = words_to_bytes_be(&data);

        // use crate::utils::print_bytes_hex;
        // print_bytes_hex(&data);

        return ;
    }

    if start_stub {
        use env_logger::Env;
        use crate::utils::wait_for_new_debugger_at_port;

        let memsize = 1024*1024;
        let port    = 9999u16;

        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

        let riscv32_dbg = wait_for_new_debugger_at_port(memsize, port);

        riscv32_dbg.custom_gdb_event_loop_thread();
        // riscv32_dbg.default_gdb_event_loop_thread();

        return ;
    }

    if write_elf {
        use crate::utils::encode_to_elf;

        let linker = "riscv32-unknown-linux-gnu-ld";
        let execfile = "main";
        let objectfile = "main.o";
        let srcfile = args[2];

        let f = std::fs::read_to_string(srcfile).unwrap();

        encode_to_elf(&f, objectfile).unwrap();

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

    if decode_binary {
        use crate::lang::ext::InstructionFormat;
        use crate::lang::ext::Immediate;

        let n = args[2].trim().replace("0x", "");

        let word = u32::from_str_radix(&n, 16).expect("Invalid instruction");

        let iformat = InstructionFormat::decode(word);

        println!("{:?}", iformat);

        if let Some(iformat) = iformat {
            match iformat {
                InstructionFormat::B { imm, .. } => {
                    println!("Immediate: {:x}", imm.decode());
                },
                // InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode } => todo!(),
                // InstructionFormat::I { imm, rs1, funct3, rd, opcode } => todo!(),
                // InstructionFormat::S { imm, rs2, rs1, funct3, opcode } => todo!(),
                // InstructionFormat::U { imm, rd, opcode } => todo!(),
                // InstructionFormat::J { imm, rd, opcode } => todo!(),
                _ => todo!(),
            }
        }

        return ;
    }

    if decode_text {
        use crate::utils::encode_to_word;

        let code = args[2];

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

fn usage() {
    println!("Usage");
    println!("  cargo run -- [ --build    | -b ] file.s");
    println!("  cargo run -- [ --debugger | -d ]");
    println!("  cargo run -- [ --decode-bin    ] 0x00001117");
    println!("  cargo run -- [ --decode-text   ] \"addi a2,a1,3\"");
    println!("  cargo run -- [ --elf      | -e ] file.s");
}
