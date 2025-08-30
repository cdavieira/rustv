pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod assembler;
pub mod memory;
pub mod cpu;
pub mod machine;
pub mod elf;
pub mod utils;

#[cfg(test)]
mod tests {
    mod gas {
        use crate::cpu::{SimpleCPU, CPU};
        use crate::memory::{BasicMemory, Memory};
        use crate::spec::AssemblySectionName;
        use crate::tokenizer::Tokenizer;
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        use crate::assembler::Assembler;
        use crate::machine::Machine;
        use super::super::*;

        // Tokenization of sources

        #[test]
        fn tokenize_ignore_comments(){
            let code = "
                //this is gonna be great\n

                //awesome
            ";
            let expected: Vec<String> = vec![];
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_words(){
            let code = "abc paulista oloco";
            let expected: Vec<String> = code
                .split_whitespace()
                .map(|s| String::from(s)).collect();
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_commas_parenthesis(){
            let code = "a, b, c(d, e(f)g, h";
            let expected: Vec<String> = code.chars()
                .filter(|ch| !matches!(ch, ' ' | '\n') )
                .map(|ch| String::from(ch))
                .collect();
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_labels(){
            let code = "
                main:
                loop2:
            ";
            let expected: Vec<String> = vec![String::from("main:"), String::from("loop2:")];
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_sections(){
            let code = "
                .globl
                .text
            ";
            let expected: Vec<String> = vec![String::from(".globl"), String::from(".text")];
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_numbers(){
            let code = "-1 +3 -66 1000";
            let expected: Vec<String> = code.split_whitespace().map(|s| String::from(s)).collect();
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_hex_offset(){
            let code = "sw 5,0x3(6)";
            let expected = [
                "sw", "5", ",", "0x3", "(", "6", ")"
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }
    
        #[test]
        fn tokenize_binary_ops(){
            let code = "
                loop:   beq  x11, x0,  exit
                        add  x11, x5 + 5,  x0
                        beq  x0,  3 + -9,  loop
            ";
            let expected = [
                "loop:", "beq", "x11", ",", "x0", ",", "exit",
                "add", "x11", ",", "x5", "+", "5", ",", "x0",
                "beq", "x0", ",", "3", "+", "-9", ",", "loop",
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        // #[ignore]
        //add, and, or (R), lui (U), jal (J), addi, andi, ori, lw (I), sw (S), beq, blt (B)
        fn tokenize_rv32i_subset0(){
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
                "addi", "sp", ",", "sp", ",", "16",
                "andi", "sp", ",", "sp", ",", "16",
                "ori", "sp", ",", "sp", ",", "16",
                "sw", "t0", ",", "3", "(", "t1", ")",
                "beq", "t1", ",", "t2", ",", "0x900",
                "blt", "t1", ",", "t2", ",", "0x900",
                "lui", "t3", ",", "25",
                "add", "t3", ",", "t2", ",", "t1",
                "or", "t3", ",", "t2", ",", "t1",
                "and", "t3", ",", "t2", ",", "t1",
                "jal", "t4", ",", "0x1000",
                "lw", "ra", ",", "-12", "(", "sp", ")",
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_strings(){
            let code = "\"isso ai\"  \"esse \\\"cara\\\"\"";
            let expected = [
                "\"isso ai\"", "\"esse \\\"cara\\\"\"",
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
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
                "loop:", "beq", "x11", ",", "x0", ",", "exit",
                "add", "x11", ",", "x5", ",", "x0",
                "beq", "x0", ",", "x0", ",", "loop",
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
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
                ".text",
                ".globl", "main",
                "main:",
                "addi", "sp", ",", "sp", ",", "-16",
                "sw", "ra", ",", "12", "(", "sp", ")",
                "sw", "s0", ",", "8", "(", "sp", ")",
                "addi", "s0", ",", "sp", ",", "16",
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
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
                ".text",
                ".globl", "main",
                "main:",
                "li", "a0", ",", "0",
                "lw", "ra", ",", "12", "(", "sp", ")",
                "lw", "s0", ",", "8", "(", "sp", ")",
                "addi", "sp", ",", "sp", ",", "16",
                "ret",
            ].map(|s| String::from(s));
            let mut tokenizer = syntax::gas::Tokenizer;
            let res: Vec<String> = tokenizer.get_tokens(code);
            assert_eq!(res, expected);
        }



        // Binary encoding of instructions

        fn encode_instructions(code: &str) -> Vec<u32> {
            let mut tokenizer = syntax::gas::Tokenizer;
            let lexer = syntax::gas::Lexer;
            let parser = syntax::gas::Parser;
            let assembler = syntax::gas::Assembler;
            let tokens = tokenizer.get_tokens(code);
            let lexemes = lexer.parse(tokens);
            let mut parser_output = parser.parse(lexemes);
            let text = parser_output.get_sections().into_iter().find(|stat| match stat.name {
                AssemblySectionName::TEXT => true,
                _ => false
            }).unwrap();
            assembler.to_words(text).data
        }

        fn encode_single_instruction(code: &str) -> u32 {
            *encode_instructions(code).get(0).unwrap()
        }

        #[test]
        fn encode_addi() {
            let code = "addi  sp, sp, 16";
            let expected: u32 = 0x01010113;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_sw() {
            let code = "sw t0,3(t1)";
            let expected: u32 = 0x005321a3;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        #[ignore]
        //OBS: the 'as' assembler uses two instructions to encode bne: 'beq' followed by a 'j'.
        //For this reason, the offset given in the instruction isn't the exact same generated in
        //the bytecode, because the encoded offset jumps PC to the position of the consecutive 'j'
        //instruction. For that reason, in this test the offset 10 gets encoded as 8.
        fn encode_bne() {
            let code = "bne t1,t2,10";
            let expected: u32 = 0x00731463;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_lui() {
            let code = "lui t3,25";
            let expected: u32 = 0x00019e37;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_lw() {
            let code = "lw ra,-12(sp)";
            let expected: u32 = 0xff412083;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_add() {
            let code = "add t3,t2,t1";
            let expected: u32 = 0x00638e33;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }

        #[test]
        fn encode_ret() {
            let code = "ret";
            let expected: Vec<u32> = vec![0x00008067];
            let res = encode_instructions(code);
            assert_eq!(res, expected, "LEFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_li_short() {
            let code = "li a0, 93";
            let expected: Vec<u32> = vec![0x05d00513];
            let res = encode_instructions(code);
            assert_eq!(res, expected, "LEFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_li_long() {
            let code = "li t0, 10000";
            let expected: Vec<u32> = vec![0x000022b7, 0x71028293];
            let res = encode_instructions(code);
            assert_eq!(res, expected, "LEFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        fn encode_exit() {
            let code = "
                li a7, 93 // Linux syscall: exit
                li a0, 0  // return code 0
                ecall     // make the syscall
            ";
            let expected: Vec<u32> = vec![0x05d00893, 0x00000513, 0x00000073];
            let res = encode_instructions(code);
            assert_eq!(res, expected, "LEFT: {res:?}, RIGHT: {expected:?}");
        }

        #[test]
        #[ignore]
        fn encode_jal() {
            // let code = "jal t4,0x900";
            let code = "jal t4,18";
            let expected: u32 = 0x00000eef;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }



        // CPU
        #[test]
        fn cpu_rw_all_gpr() {
            let mut cpu = SimpleCPU::new();
            let default_value = 1000u32;
            for reg in 0..32 {
                cpu.write(reg, default_value);
            }
            for reg in 0..32 {
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



        // Memory
        #[test]
        fn memory_rw_byte() {
            let mut memory = BasicMemory::new();
            let values = [1u8, 2u8, 3u8, 4u8];
            memory.reserve_bytes(values.len());
            for (idx, value) in values.into_iter().enumerate() {
                memory.write_byte(idx, value);
                assert_eq!(memory.read_byte(idx), value, "Error reading byte at {}", idx);
            }
        }

        #[test]
        fn memory_rw_word() {
            let mut memory = BasicMemory::new();
            let values = [1u32, 2u32, 3u32, 4u32];
            memory.reserve_words(values.len());
            for (idx, value) in values.into_iter().enumerate() {
                memory.write_word(idx*4, value);
                assert_eq!(memory.read_word(idx*4), value, "Error reading byte at {}", idx);
            }
        }

        #[test]
        fn memory_rw_bytes_from_word() {
            let mut memory = BasicMemory::new();
            let word = u32::from_be_bytes([0, 0, 0, 100u8]); //100
            memory.reserve_words(1);
            memory.write_word(0, word);
            assert_eq!(memory.read_word(0), word);
        }

        #[test]
        fn memory_rw_bytes() {
            let mut memory = BasicMemory::new();
            let values = [1u8, 2u8, 3u8, 4u8];
            memory.reserve_bytes(values.len());
            memory.write_bytes(0, &values.to_vec());
            assert_eq!(memory.bytes(), values);
        }

        #[test]
        fn memory_rw_words() {
            let mut memory = BasicMemory::new();
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
            let words = encode_instructions(code);
            let mut m = machine::SimpleMachine::new(&words);
            m.decode();
            assert!(m.assert_reg(17u32, 93));
        }

        #[test]
        fn machine_test1() {
            let code = "
                li a7, 93    // Linux syscall: exit
                li a0, 1000  // return code 0
            ";
            let words = encode_instructions(code);
            let mut m = machine::SimpleMachine::new(&words);
            m.decode();
            m.decode();
            assert!(m.assert_reg(17u32, 93));
            assert!(m.assert_reg(10u32, 1000));
        }



        // Test elf R/W
        #[test]
        fn elf_write() {
            let filename = "test_elf_write.o";
            let code = "
                li a7, 93 // Linux syscall: exit
                li a0, 0  // return code 0
                ecall     // make the syscall
            ";
            let words = encode_instructions(code);
            let m = machine::SimpleMachine::new(&words);

            let write_res = elf::write_elf(filename, m.bytes());
            let rem_res = std::fs::remove_file(filename);

            assert!(write_res.is_ok() && rem_res.is_ok());
        }

        #[test]
        fn elf_read() {
            let filename = "test_elf_read.o";
            let code = "
                li a7, 93 // Linux syscall: exit
                li a0, 0  // return code 0
                ecall     // make the syscall
            ";
            let words_written = encode_instructions(code);
            let m = machine::SimpleMachine::new(&words_written);

            let write_res = elf::write_elf(filename, m.bytes());
            let read_res = elf::read_elf(filename);
            let rem_res = std::fs::remove_file(filename);

            assert!(write_res.is_ok());
            assert!(read_res.is_ok());
            assert!(rem_res.is_ok());
            if let Ok(words_read) = read_res {
                assert_eq!(words_written, words_read);
            }
        }
    }
}
