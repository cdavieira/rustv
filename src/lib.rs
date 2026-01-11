pub mod assembler;
pub mod lexer;
pub mod parser;
pub mod streamreader;
pub mod syntax;
pub mod tokenizer;
pub mod utils;
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
    pub mod dwarfwriter;
    pub mod elfreader;
    pub mod elfwriter;
}

#[cfg(test)]
mod tests {
    mod gas {
        use super::super::*;
        use crate::assembler::AssemblerTools;
        use crate::emu::{
            cpu::CPU, cpu::SimpleCPU, machine::Machine, machine::SimpleMachine, memory::Memory,
            memory::SimpleMemory,
        };
        use crate::lang::highassembly::{Register, SectionName};
        use crate::lang::lowassembly::DataEndianness;
        use crate::lexer::Lexer;
        use crate::obj::{elfreader::ElfReader, elfwriter::ElfWriter};
        use crate::streamreader::{CharStreamReader, Position, StreamReader};
        use crate::utils::{
            build_code_repr, encode_to_word, encode_to_words, new_machine_from_tools,
            set_remaining_bits,
        };

        // Custom iterator
        #[test]
        fn char_iterator() {
            let buf = "h\nm a";
            let mut it = CharStreamReader::new(buf.chars(), '\n');
            let states = [
                (
                    Some('h'),
                    Some(Position::new(0, 0, 0)),
                    Some('\n'),
                    Some(Position::new(1, 0, 1)),
                ),
                (
                    Some('\n'),
                    Some(Position::new(1, 0, 1)),
                    Some('m'),
                    Some(Position::new(2, 1, 0)),
                ),
                (
                    Some('m'),
                    Some(Position::new(2, 1, 0)),
                    Some(' '),
                    Some(Position::new(3, 1, 1)),
                ),
                (
                    Some(' '),
                    Some(Position::new(3, 1, 1)),
                    Some('a'),
                    Some(Position::new(4, 1, 2)),
                ),
                (Some('a'), Some(Position::new(4, 1, 2)), None, None),
                (None, None, None, None),
            ];
            for (index, state) in states.iter().enumerate() {
                assert_eq!(
                    it.current_token(),
                    state.0,
                    "current token failed at {}",
                    index
                );
                assert_eq!(
                    it.current_position(),
                    state.1,
                    "current position failed at {}",
                    index
                );
                assert_eq!(it.next_token(), state.2, "next token failed at {}", index);
                assert_eq!(
                    it.next_position(),
                    state.3,
                    "next position failed at {}",
                    index
                );
                it.advance_and_read();
            }
        }

        // Tokenization of sources

        #[test]
        fn tokenize_ignore_comments() {
            let code = "
                //this is gonna be great\n

                //awesome
            ";
            let expected: Vec<String> = vec![];
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_words() {
            let code = "abc paulista oloco";
            let expected: Vec<String> = code.split_whitespace().map(|s| String::from(s)).collect();
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_commas_parenthesis() {
            let code = "a, b, c(d, e(f)g, h";
            let expected: Vec<String> = code
                .chars()
                .filter(|ch| !matches!(ch, ' ' | '\n'))
                .map(|ch| String::from(ch))
                .collect();
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_labels() {
            let code = "
                main:
                loop2:
            ";
            let expected: Vec<String> = vec![String::from("main:"), String::from("loop2:")];
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_sections() {
            let code = "
                .globl
                .text
            ";
            let expected: Vec<String> = vec![String::from(".globl"), String::from(".text")];
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_numbers() {
            let code = "-1 +3 -66 1000";
            let expected: Vec<String> = code.split_whitespace().map(|s| String::from(s)).collect();
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_hex_offset() {
            let code = "sw 5,0x3(6)";
            let expected = ["sw", "5", ",", "0x3", "(", "6", ")"].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_binary_ops() {
            let code = "
                loop:   beq  x11, x0,  exit
                        add  x11, x5 + 5,  x0
                        beq  x0,  3 + -9,  loop
            ";
            let expected = [
                "loop:", "beq", "x11", ",", "x0", ",", "exit", "add", "x11", ",", "x5", "+", "5",
                ",", "x0", "beq", "x0", ",", "3", "+", "-9", ",", "loop",
            ]
            .map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        //add, and, or (R), lui (U), jal (J), addi, andi, ori, lw (I), sw (S), beq, blt (B)
        #[test]
        fn tokenize_rv32i_subset0() {
            let code = "
                addi    sp, sp, 16
                andi    sp, sp, 16
                ori     sp, sp, 16
                sw      t0,3(t1)
                beq     t1,t2,0x900
                blt     t1,t2,0x900
                lui     t3,25
                add     t3,t2,t1
                or      t3,t2,t1
                and     t3,t2,t1
                jal     t4,0x1000
                lw      ra, -12(sp)
            ";
            let expected = [
                "addi", "sp", ",", "sp", ",", "16", "andi", "sp", ",", "sp", ",", "16", "ori",
                "sp", ",", "sp", ",", "16", "sw", "t0", ",", "3", "(", "t1", ")", "beq", "t1", ",",
                "t2", ",", "0x900", "blt", "t1", ",", "t2", ",", "0x900", "lui", "t3", ",", "25",
                "add", "t3", ",", "t2", ",", "t1", "or", "t3", ",", "t2", ",", "t1", "and", "t3",
                ",", "t2", ",", "t1", "jal", "t4", ",", "0x1000", "lw", "ra", ",", "-12", "(",
                "sp", ")",
            ]
            .map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_strings() {
            let code = "\"isso ai\"  \"esse \\\"cara\\\"\"";
            let expected = ["\"isso ai\"", "\"esse \\\"cara\\\"\""].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_code0() {
            let code = "
                loop:   beq  x11, x0,  exit
                        add  x11, x5,  x0
                        beq  x0,  x0,  loop
            ";
            let expected = [
                "loop:", "beq", "x11", ",", "x0", ",", "exit", "add", "x11", ",", "x5", ",", "x0",
                "beq", "x0", ",", "x0", ",", "loop",
            ]
            .map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_code1() {
            let code = "
                    .text
                    .globl main
                main:
                    addi sp, sp, -16
                    sw   ra, 12(sp)
                    sw   s0, 8(sp)
                    addi s0, sp, 16
            ";
            let expected = [
                ".text", ".globl", "main", "main:", "addi", "sp", ",", "sp", ",", "-16", "sw",
                "ra", ",", "12", "(", "sp", ")", "sw", "s0", ",", "8", "(", "sp", ")", "addi",
                "s0", ",", "sp", ",", "16",
            ]
            .map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_code2() {
            let code = "
                    .text
                    .globl main //this is gonna be great\n
                main:
                    li   a0, 0

                    lw   ra, 12(sp)
                    lw   s0, 8(sp)
                    addi sp, sp, 16
                    ret
            ";
            let expected = [
                ".text", ".globl", "main", "main:", "li", "a0", ",", "0", "lw", "ra", ",", "12",
                "(", "sp", ")", "lw", "s0", ",", "8", "(", "sp", ")", "addi", "sp", ",", "sp", ",",
                "16", "ret",
            ]
            .map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Lexer;
            let res: Vec<String> = tokenizer
                .get_tokens(code)
                .into_iter()
                .map(|token| token.0)
                .collect();
            assert_eq!(res, expected);
        }

        // TODO: Lexer tests

        // Bits utils

        #[test]
        fn bits_set_remaining_bits() {
            let n = 0b101;
            let m = set_remaining_bits(n, 2, 1);
            assert_eq!(m, 0b11111111_11111111_11111111_11111101);
        }

        // Binary encoding of instructions

        #[test]
        fn encode_addi() {
            let code = "addi  sp, sp, 16";
            let expected: u32 = 0x01010113;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_sw() {
            let code = "sw t0,3(t1)";
            let expected: u32 = 0x005321a3;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        //OBS: when working with labels, it becomes trickier to think about the <offset>, since a
        //instruction such as `bne t1,t2,label1` will become `bne t1,t2,rel_off`, where `rel_off`
        //is the difference between the address of the `bne` line and the address associated with
        //the label `label1`. Therefore, the semantics of this instruction with labels is slightly
        //different from the semantics with numbers, as numbers are actual offsets, while labels
        //are absolute addresses which then get converted to offsets by the compiler
        #[test]
        fn encode_bne() {
            let code = "bne t1,t2,8";
            let expected: u32 = 0x00731463;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        //OBS: the immediate used is going to be placed in the rd[31:12] bit range of the rd
        //register
        #[test]
        fn encode_lui() {
            let code = "lui t3,25";
            let expected: u32 = 0x00019e37;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_lw() {
            let code = "lw ra,-12(sp)";
            let expected: u32 = 0xff412083;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_add() {
            let code = "add t3,t2,t1";
            let expected: u32 = 0x00638e33;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_ret() {
            let code = "ret";
            let expected: Vec<u32> = vec![0x00008067];
            let res = encode_to_words(code);
            assert_eq!(res, expected, "LeFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_li_short() {
            let code = "li a0, 93";
            let expected: Vec<u32> = vec![0x05d00513];
            let res = encode_to_words(code);
            assert_eq!(res, expected, "LeFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_li_long() {
            let code = "li t0, 10000";
            let expected: Vec<u32> = vec![0x000022b7, 0x71028293];
            let res = encode_to_words(code);
            assert_eq!(res, expected, "LeFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_exit_0() {
            let code = "
                li a7, 93 // Linux syscall: exit
                li a0, 0  // return code 0
                ecall     // make the syscall
            ";
            let expected: Vec<u32> = vec![0x05d00893, 0x00000513, 0x00000073];
            let res = encode_to_words(code);
            assert_eq!(res, expected, "LeFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_exit_1000() {
            let code = "
                li a7, 93    // Linux syscall: exit
                li a0, 1000  // return code 1000
                ecall        // make the syscall
            ";
            let expected: Vec<u32> = vec![0x05d00893, 0x3e800513, 0x00000073];
            let res = encode_to_words(code);
            assert_eq!(res, expected, "LeFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_jal() {
            let code = "
                    .section .text
                    .globl _start
            _start:
                    jal t2,_mylabel
                    nop
                    nop
            _mylabel:
                    nop
            ";
            let expected: u32 = 0x00c003ef;
            let res = encode_to_word(code);
            assert_eq!(res, expected, "LeFT: {res:x}, RIGHT: {expected:x}");
        }

        // Test Endianness
        #[test]
        fn endianness_rw_bytes_to_word() {
            let val = 0x10080u32;
            let values = DataEndianness::break_word_into_bytes(val, DataEndianness::Be);
            let value_be = DataEndianness::build_word_from_bytes(values, DataEndianness::Be);
            assert_eq!(values, [0x0, 0x01, 0x00, 0x80]);
            assert_eq!(value_be, val);

            let val = 0x00001117;
            let values = DataEndianness::break_word_into_bytes(val, DataEndianness::Be);
            let value_be = DataEndianness::build_word_from_bytes(values, DataEndianness::Be);
            assert_eq!(values, [0x0, 0x0, 0x11, 0x17]);
            assert_eq!(value_be, val);
        }

        // CPU
        #[test]
        fn cpu_rw_all_gpr() {
            let mut cpu = SimpleCPU::new();
            let default_value = 1000u32;
            for reg in 0..32 {
                cpu.write(reg, default_value);
            }
            assert_eq!(cpu.read(0), 0);
            for reg in 1..32 {
                assert_eq!(cpu.read(reg), default_value);
            }
        }

        #[test]
        fn cpu_rw_pc() {
            let mut cpu = SimpleCPU::new();
            let default_value = 1000usize;
            cpu.write_pc(default_value);
            assert_eq!(cpu.read_pc(), default_value);
        }

        #[test]
        fn cpu_write_all() {
            let mut cpu = SimpleCPU::new();
            let default_value = 1000u32;
            let values: Vec<u32> = (0..32).map(|_| default_value).collect();
            cpu.write_all(values, default_value as usize);
            assert_eq!(cpu.read(0), 0);
            assert_eq!(cpu.read_pc(), default_value as usize);
            for reg in 1..32 {
                assert_eq!(cpu.read(reg), default_value);
            }
        }

        // Memory
        #[test]
        fn memory_rw_byte() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let values = [1u8, 2u8, 3u8, 4u8];
            memory.reserve_bytes(values.len());
            for (idx, value) in values.into_iter().enumerate() {
                memory.write_byte(idx, value);
                assert_eq!(
                    memory.read_byte(idx),
                    value,
                    "Error reading byte at {}",
                    idx
                );
            }
        }

        #[test]
        fn memory_rw_word() {
            let mut memory = SimpleMemory::new(DataEndianness::Le);
            let values = [1u32, 2u32, 3u32, 4u32];
            memory.reserve_words(values.len());
            for (idx, value) in values.into_iter().enumerate() {
                memory.write_word(idx * 4, value);
                assert_eq!(
                    memory.read_word(idx * 4),
                    value,
                    "Error reading byte at {}",
                    idx
                );
            }
        }

        #[test]
        fn memory_rw_bytes_from_word() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let word = u32::from_be_bytes([0, 0, 0, 100u8]); //100
            memory.reserve_words(1);
            memory.write_word(0, word);
            assert_eq!(memory.read_word(0), word);
        }

        #[test]
        fn memory_rw_bytes() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let values = [1u8, 2u8, 3u8, 4u8];
            memory.reserve_bytes(values.len());
            memory.write_bytes(0, &values.to_vec(), DataEndianness::Be);
            assert_eq!(memory.bytes(), values);
        }

        #[test]
        fn memory_rw_words() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let values = [1u32, 2u32, 3u32, 4u32];
            memory.reserve_words(values.len());
            memory.write_words(0, &values.to_vec());
            assert_eq!(memory.words(), values);
        }

        // Test Machine
        #[test]
        fn machine_test0() {
            let code = "
                li a7, 93    // Linux syscall: exit
            ";
            let words = encode_to_words(code);
            let mut m = SimpleMachine::from_words(&words, DataEndianness::Be);
            m.decode().unwrap();
            assert!(m.assert_reg(17u32, 93));
        }

        #[test]
        fn machine_test1() {
            let code = "
                li a7, 93    // Linux syscall: exit
                li a0, 1000  // return code 0
            ";
            let words = encode_to_words(code);
            let mut m = SimpleMachine::from_words(&words, DataEndianness::Be);
            m.decode().unwrap();
            m.decode().unwrap();
            assert!(m.assert_reg(17u32, 93));
            assert!(m.assert_reg(10u32, 1000));
        }

        // Test ISA
        // TODO: test more complex cases (negative offsets, sections in different orders than the
        // usual, jumps to non-existing labels, ...)
        fn isa_rvi32_mach_only_text(code: &str) -> SimpleMachine {
            let words = encode_to_words(code);
            let mut m = SimpleMachine::from_words(&words, DataEndianness::Be);
            for _word in words {
                m.decode().unwrap();
            }
            m
        }

        fn isa_rvi32_mach(code: &str) -> (SimpleMachine, AssemblerTools) {
            let tools = build_code_repr(code);
            let mut m = new_machine_from_tools(&tools);
            let text = tools.text_section_words();
            let text_len = text.len();
            for _ in 0..text_len {
                m.decode().unwrap();
            }
            (m, tools)
        }

        fn isa_rvi32_mach_deterministic(
            code: &str,
            n_decodes: u32,
        ) -> (SimpleMachine, AssemblerTools) {
            let tools = build_code_repr(code);
            let mut m = new_machine_from_tools(&tools);
            for _ in 0..n_decodes {
                m.decode().unwrap();
            }
            (m, tools)
        }

        fn isa_rvi32_mach_until_exit(code: &str) -> (SimpleMachine, AssemblerTools) {
            let tools = build_code_repr(code);
            let mut m = new_machine_from_tools(&tools);
            loop {
                match m.decode() {
                    Ok(state) => match state {
                        emu::machine::MachineState::Exit(_status) => break,
                        emu::machine::MachineState::Ok => {}
                    },
                    Err(_err) => break,
                }
            }
            (m, tools)
        }

        #[test]
        fn isa_rvi32_add() {
            let code = "
                li a2, 90
                li a3, 150
                add a1, a2, a3
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 240));
        }

        #[test]
        fn isa_rvi32_sub() {
            let code = "
                li a2, 90
                li a3, 150
                sub a1, a2, a3
            ";
            let m = isa_rvi32_mach_only_text(code);
            let n = -60;
            assert!(m.assert_reg(Register::A1.id().into(), n as u32));
        }

        #[test]
        fn isa_rvi32_and() {
            let code = "
                li a2, 6
                li a3, 2
                and a1, a2, a3
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 2));
        }

        #[test]
        fn isa_rvi32_or() {
            let code = "
                li a2, 1
                li a3, 2
                or a1, a2, a3
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 3));
        }

        #[test]
        fn isa_rvi32_xor() {
            let code = "
                li a2, 90
                xor a2, a2, a2
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 0));
        }

        #[test]
        fn isa_rvi32_sll() {
            let code = "
                li a2, 4
                li a3, 2
                sll a1, a2, a3
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 16));
        }

        #[test]
        fn isa_rvi32_srl() {
            let code = "
                li a2, 4
                li a3, 2
                srl a1, a2, a3
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 1));
        }

        #[test]
        fn isa_rvi32_jalr() {
            let code = "
                li a2, 4
                jalr ra, a2, 8
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::RA.id().into(), 8));
            assert!(m.assert_pc(12));
        }

        #[test]
        fn isa_rvi32_addi() {
            let code = "
                li a2, 4
                addi a1, a2, 8
                addi a3, a2, 888
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 12));
            assert!(m.assert_reg(Register::A3.id().into(), 892));
        }

        #[test]
        fn isa_rvi32_andi() {
            let code = "
                li a2, 6
                andi a1, a2, 4
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 4));
        }

        #[test]
        fn isa_rvi32_xori() {
            let code = "
                li a2, 7
                xori a1, a2, 4
            ";
            let m = isa_rvi32_mach_only_text(code);
            assert!(m.assert_reg(Register::A1.id().into(), 3));
        }

        #[test]
        fn isa_rvi32_sw() {
            let code = "
                .section .data
                var1: .word 0x4
                var2: .word 0xa

                .section .text
                    la t1, var1
                    li t2, 100
                    sw t2, 4(t1)
            ";
            let (m, tools) = isa_rvi32_mach(code);
            let data_section = tools
                .sections
                .get(".data")
                .expect("missing start address for data section");
            let start_addr = data_section.address + 4;
            let expected = vec![100u32];
            assert!(m.assert_memory_words(start_addr, expected.len(), &expected));
        }

        #[test]
        fn isa_rvi32_sb() {
            let code = "
                .section .data
                var1: .byte 0x4 0xa
                // var2: .byte 0xa

                .section .text
                    la t1, var1
                    li t2, 100
                    sb t2, 1(t1)
            ";
            let (m, tools) = isa_rvi32_mach(code);
            let data_section = tools
                .sections
                .get(".data")
                .expect("missing start address for data section");
            let start_addr = data_section.address + 1;
            let expected = vec![100u8];
            assert!(m.assert_memory_bytes(start_addr, expected.len(), &expected, 1));
        }

        // OBS: LW syntax is rd, off, rs
        // so 'lw t2, t1' becomes
        // rd = t2
        // off = t1
        // rs = 0 (default)
        #[test]
        fn isa_rvi32_lw() {
            let code = "
                .section .data
                var1: .word 0x4
                .section .text
                    la t1, var1
                    lw t2, 0(t1)
            ";
            let (m, _) = isa_rvi32_mach(code);
            let reg = Register::T2.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            assert_eq!(reg, 4);
        }

        #[test]
        fn isa_rvi32_lb() {
            let code = "
                .section .data
                var1: .byte -0x1
                .section .text
                    la t1, var1
                    lb t2, 3(t1)
            ";
            let (m, _) = isa_rvi32_mach(code);
            let reg = Register::T2.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            let val = 0b1111_1111;
            assert_eq!(reg, val);
        }

        #[test]
        fn isa_rvi32_beq() {
            let code = "
                .section .text
                _start:
                    li t1, 2
                    li t2, 2
                    beq t2, t1, _continue
                    li t3, 4
                _continue:
                    li t4, 5
            ";
            let (m, _) = isa_rvi32_mach_deterministic(code, 4);
            let reg = Register::T3.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            assert_eq!(reg, 0);
        }

        #[test]
        fn isa_rvi32_bne() {
            let code = "
                .section .text
                _start:
                    li t1, 4
                    li t2, 2
                    bne t2, t1, _continue
                    li t3, 4
                _continue:
                    li t4, 5
            ";
            let (m, _) = isa_rvi32_mach_deterministic(code, 4);
            let reg = Register::T3.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            assert_eq!(reg, 0);
        }

        #[test]
        fn isa_rvi32_blt() {
            let code = "
                .section .text
                _start:
                    li t1, 4
                    li t2, 2
                    blt t2, t1, _continue
                    li t3, 4
                _continue:
                    li t4, 5
            ";
            let (m, _) = isa_rvi32_mach_deterministic(code, 4);
            let reg = Register::T3.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            assert_eq!(reg, 0);
        }

        #[test]
        fn isa_rvi32_bge() {
            let code = "
                .section .text
                _start:
                    li t1, 2
                    li t2, 4
                    bge t2, t1, _continue
                    li t3, 4
                _continue:
                    li t4, 5
            ";
            let (m, _) = isa_rvi32_mach_deterministic(code, 4);
            let reg = Register::T3.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            assert_eq!(reg, 0);
        }

        #[test]
        fn isa_rvi32_lui() {
            let code = "
                lui t1, 0x10000
            ";
            let m = isa_rvi32_mach_only_text(code);
            let reg = Register::T1.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            let val = 0x10000 << 12;
            assert_eq!(reg, val);
        }

        #[test]
        fn isa_rvi32_auipc() {
            let code = "
                .section .text
                _start:
                    addi x0, x0, 0
                    addi x0, x0, 0
                    auipc t1, 0x10000
            ";
            let (m, _) = isa_rvi32_mach(code);
            let reg = Register::T1.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg];
            let val = 8 + (0x10000 << 12);
            assert_eq!(reg, val);
        }

        #[test]
        fn isa_rvi32_jal() {
            let code = "
                .section .text
                _start:
                    nop
                    jal t1, mylabel2
                    li t2, 4
                mylabel2:
                    li t3, 4
            ";
            let (m, _) = isa_rvi32_mach_deterministic(code, 3);
            let t1 = Register::T1.id() as usize;
            let t2 = Register::T2.id() as usize;
            let regs = m.read_registers();
            assert_eq!(regs[t2], 0);
            assert_eq!(regs[t1], 8);
        }

        #[test]
        fn isa_m_mul() {
            let code = "
                li t1, 3
                li t2, 4
                li t3, -4
                mul t4, t1, t2
                mul t5, t2, t3
            ";
            let m = isa_rvi32_mach_only_text(code);
            let t4 = Register::T4.id() as usize;
            let t5 = Register::T5.id() as usize;
            let regs = m.read_registers();
            let t4 = regs[t4] as u32;
            let t5 = regs[t5] as i32;
            assert_eq!(t4, 12);
            assert_eq!(t5, -16);
        }

        // Test programs
        #[test]
        fn program_funccall() {
            let code = "
                        .globl _start
                        .section .text
                _start:
                        li t1, 3
                        li a0, 1
                        li a1, -2
                        jal ra, myfunc
                        li a7, 93
                        li a0, 1000
                        ecall
                myfunc:
                        add t1, a0, a1
                        ret
            ";
            let (m, _) = isa_rvi32_mach(code);
            let reg = Register::T1.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg] as i32;
            let val = -1;
            assert_eq!(reg, val);
        }

        #[test]
        fn program_stack() {
            let code = "
                    .globl _start
                    .section .text
                _start:
                    // prepare stack
                    la sp, 64(stacktop)
                    jal ra, myfunc
                    li a7, 93
                    li a0, 1000
                    ecall
                    nop
                    nop
                    nop
                myfunc:
                    // allocate new function frame
                    addi sp, sp, -16
                    // save 'registers' in previous stack
                    sw ra, 12(sp) // 12(sp) - 16(sp) = ra
                    sw fp, 8(sp) // 8(sp) - 12(sp) = fp
                    addi fp, sp, 16 // fp -> stack base
                    la a1, -4(fp) // a1 = ra
                    ret
                    .section .data
                stacktop:
                    .skip 64
            ";
            let (m, _t) = isa_rvi32_mach(code);
            let reg = Register::A1.id() as usize;
            let regs = m.read_registers();
            let reg = regs[reg] as i32;
            let val = 124;
            assert_eq!(reg, val);
        }

        #[test]
        fn program_rec_fcall() {
            let code = "
                        .globl _start
                        .section .text
                _start:
                        la sp, 64(stacktop)
                        mv t3, sp
                        li a2, 1
                        jal ra, myrecfunc
                        li a7, 93
                        li a0, 1000
                        ecall
                myrecfunc:
                        // allocate new function frame and function pointer
                        addi sp, sp, -16
                        sw ra, 12(sp)    // 12(sp) - 16(sp) = ra
                        sw fp, 8(sp)     // 8(sp) - 12(sp) = fp
                        addi fp, sp, 16  // fp -> stack base
                        // Do function work
                        beq a2, 0, end
                        addi a2, a2, -1
                        jal ra, myrecfunc
                end:
                        // Popping function frame
                        lw ra, 12(sp)
                        lw fp, 8(sp)
                        addi sp, sp, 16
                        ret

                        .section .data
                stacktop:
                    .skip 64
            ";
            let n_executions = 28;
            let (m, _t) = isa_rvi32_mach_deterministic(code, n_executions);
            let a2 = Register::A2.id() as usize;
            let t3 = Register::T3.id() as usize;
            let sp = Register::SP.id() as usize;
            let regs = m.read_registers();
            let a2reg = regs[a2] as i32;
            let t3reg = regs[t3] as i32;
            let spreg = regs[sp] as i32;
            assert_eq!(a2reg, 0);
            assert_eq!(t3reg, spreg);
        }

        #[test]
        fn program_selection_sort() {
            let code = "
                        .globl _start
                        .section .text
                _start:
                        la t0, myvector
                        li  a2, 3
                        addi a5, a2, -1
                loop:
                        loop_init_list:
                        li  t1, 0
                        li  a3, 0

                        loop_condition:
                        beq t1, a5, loop_end

                        loop_body:

                        //inner_loop:
                                inner_loop_init_list:
                                        mv   t2, t1
                                        addi t2, t2, 1            // t2 = t1 + 1
                                        mv  a4, a3
                                        addi  a4, a4, 4           // a4 = a3 + 4

                                inner_loop_condition:
                                        beq t2, a2, inner_loop_end

                                inner_loop_body:
                                        add t3, t0, a3  // t3 = t0 + a3 (myvector + 4*i)
                                        lw  t3, 0(t3)   // t3 = myvector[4*i]
                                        add t4, t0, a4    // t4 = t0 + a4 (myvector + 4*j)
                                        lw  t4, 0(t4)     // t4 = myvector[4*j]
                                        // if myvector[j] < myvector[i]
                                        //   continue;
                                        // else
                                        //   swap i'th element (t3) with j'th element (t4);
                                        blt t4, t3, inner_loop_increment
                                        mv t5, t3        // t5 = myvector[i]
                                        add t6, t0, a3   // t6 = t0 + a3 (myvector + 4*i)
                                        sw t4, 0(t6)     // *(myvector + 4*i) = t4
                                        add t6, t0, a4   // t6 = t0 + a4 (myvector + 4*j)
                                        sw t5, 0(t6)     // *(myvector + 4*j) = t5

                                inner_loop_increment:
                                        addi t2, t2, 1
                                        addi a4, a4, 4
                                        beq  x0, x0, inner_loop_condition // we've found ourselves in an (inner) loop :')

                                inner_loop_end:

                        loop_increment:
                        addi t1, t1, 1
                        addi a3, a3, 4
                        beq  x0, x0, loop_condition // we've found ourselves in a loop :')

                        loop_end:

                        li a7, 93
                        li a0, 0
                        ecall

                        .section .data
                myvector: .word 3, 5, 10
            ";
            let (m, t) = isa_rvi32_mach_until_exit(code);
            let datasection = t.sections.get(".data").unwrap();
            let varaddr = datasection.address;
            let regs = m.read_memory_words(varaddr, 3);
            assert_eq!(regs, vec![10, 5, 3]);
        }

        #[test]
        fn program_multiply_vector() {
            let code = "
                        .globl _start
                        .section .text
                _start:
                        la t0, myvector // address of 'myvector' variable
                        li t2, 3        // size of 'myvector'
                        li t3, -12      // multiplier
                        li t4, 4        // size of each element
                loop:
                        loop_init_list:
                        li  t1, 0

                        loop_condition:
                        beq t1, t2, loop_end

                        loop_body:
                        mul  t5, t1, t4 // idx to i'th element (4*i)
                        add  t5, t0, t5 // address to the i'th element (myvector + 4*i)
                        lw   t6, 0(t5)  // value of the i'th element (t6 = myvector[4*i])
                        mul  t6, t3, t6 // multiplied value (t6 = t3 * myvector[4*i])
                        sw   t6, 0(t5)  // storing multiplied value  (*(myvector + 4*i) = t6)

                        loop_increment:
                        addi t1, t1, 1
                        beq  x0, x0, loop_condition // we've found ourselves in a loop :')

                        loop_end:

                        li a7, 93
                        li a0, 0
                        ecall

                        .section .data
                myvector: .word 3, 5, 10
            ";
            let (m, t) = isa_rvi32_mach_until_exit(code);
            let f = -12;
            let datasection = t.sections.get(".data").unwrap();
            let varaddr = datasection.address;
            let regs: Vec<i32> = m
                .read_memory_words(varaddr, 3)
                .into_iter()
                .map(|reg| reg as i32)
                .collect();
            assert_eq!(regs, vec![3 * f, 5 * f, 10 * f]);
        }

        // Test elf R/W
        #[test]
        fn elf_write() {
            //li a7, 93
            //li a0, 0
            //ecall
            let words: Vec<u32> = vec![0x05d00893, 0x00000513, 0x00000073];
            let bytes = utils::words_to_bytes_le(&words);
            let filename = "test_elf_write.o";

            let mut writer = ElfWriter::new();
            writer.set_start_address(0);
            writer
                .set_section_data(SectionName::Text, bytes, 4)
                .expect("error setting text data");
            let write_res = writer.save(filename);
            let rem_res = std::fs::remove_file(filename);

            assert!(write_res.is_ok() && rem_res.is_ok());
        }

        #[test]
        fn elf_read() {
            //li a7, 93
            //li a0, 0
            //ecall
            let words: Vec<u32> = vec![0x05d00893, 0x00000513, 0x00000073];
            let bytes = utils::words_to_bytes_le(&words);
            let filename = "test_elf_read.o";

            // Writing temporary ELF file
            let mut writer = ElfWriter::new();
            writer.set_start_address(0);
            writer
                .set_section_data(SectionName::Text, bytes, 4)
                .expect("error setting text data");
            let write_res = writer.save(filename);
            assert!(write_res.is_ok());

            // Reading temporary ELF file data
            let read_io_res = std::fs::read(filename);
            if let Ok(data) = read_io_res {
                let read_res = ElfReader::new(&data, DataEndianness::Le);
                assert!(read_res.is_ok());
            } else {
                assert!(read_io_res.is_ok());
            }

            // Removing temporary ELF file from fs
            let rem_res = std::fs::remove_file(filename);
            assert!(rem_res.is_ok());
        }

        #[test]
        fn elf_rw() {
            let filename = "test_elf_rw.o";
            let code = "
                li a7, 93 // Linux syscall: exit
                li a0, 0  // return code 0
                ecall     // make the syscall
            ";

            // Encoding instructions to binary
            let words_written = encode_to_words(code);
            let bytes_written = utils::words_to_bytes_le(&words_written);

            // Saving the binary code in the ELF format
            let mut writer = ElfWriter::new();
            writer.set_start_address(0);
            writer
                .set_section_data(SectionName::Text, bytes_written.clone(), 4)
                .expect("error setting text data");
            let write_res = writer.save(filename);
            assert!(write_res.is_ok());

            // Reading/Parsing the ELF file
            let read_io_res = std::fs::read(filename);
            assert!(read_io_res.is_ok());

            // Removing the ELF file from the file system
            let rem_res = std::fs::remove_file(filename);
            assert!(rem_res.is_ok());

            // Comparing what was written to what was read
            if let Ok(data) = read_io_res {
                let read_res = ElfReader::new(&data, DataEndianness::Le);
                if let Ok(reader) = read_res {
                    let bytes_read = &reader.section(".text").unwrap().data;
                    assert_eq!(bytes_read, &bytes_written);
                } else {
                    assert!(read_res.is_ok());
                }
            } else {
                assert!(read_io_res.is_ok());
            }
        }
    }
}
