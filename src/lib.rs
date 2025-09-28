pub mod streamreader;
pub mod tokenizer;
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

#[cfg(test)]
mod tests {
    mod gas {
        use super::super::*;
        use crate::streamreader::{
            CharStreamReader,
            Position,
            StreamReader
        };
        use crate::tokenizer::Tokenizer;
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        use crate::assembler::{
            Assembler,
            AssemblerTools,
        };
        use crate::lang::highassembly::{
            Register,
            SectionName,
        };
        use crate::lang::lowassembly::{
            DataEndianness,
        };
        use crate::emu::{
            memory::Memory,
            memory::SimpleMemory,
            cpu::CPU,
            cpu::SimpleCPU,
            machine::Machine,
            machine::SimpleMachine,
        };
        use crate::utils::{
            build_code_repr,
            encode_to_word,
            encode_to_words,
            new_machine_from_tools,
        };
        use crate::obj::{
            elfwriter::ElfWriter,
            elfreader::ElfReader,
        };

        // Custom iterator
        #[test]
        fn char_iterator(){
            let buf = "h\nm a";
            let mut it = CharStreamReader::new(buf.chars(), '\n');
            let states = [
                (Some('h') , Some(Position::new(0, 0, 0)), Some('\n'), Some(Position::new(1, 0, 1))),
                (Some('\n'), Some(Position::new(1, 0, 1)), Some('m') , Some(Position::new(2, 1, 0))),
                (Some('m') , Some(Position::new(2, 1, 0)), Some(' ') , Some(Position::new(3, 1, 1))),
                (Some(' ') , Some(Position::new(3, 1, 1)), Some('a') , Some(Position::new(4, 1, 2))),
                (Some('a') , Some(Position::new(4, 1, 2)), None, None),
                (None      , None, None, None),
            ];
            for (index, state) in states.iter().enumerate() {
                assert_eq!(it.current_token(), state.0, "current token failed at {}", index);
                assert_eq!(it.current_position(), state.1, "current position failed at {}", index);
                assert_eq!(it.next_token(), state.2, "next token failed at {}", index);
                assert_eq!(it.next_position(), state.3, "next position failed at {}", index);
                it.advance_and_read();
            }
        }





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

        //add, and, or (R), lui (U), jal (J), addi, andi, ori, lw (I), sw (S), beq, blt (B)
        #[test]
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




        // TODO: Lexer tests




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






        // Memory
        #[test]
        fn memory_rw_byte() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let values = [1u8, 2u8, 3u8, 4u8];
            memory.reserve_bytes(values.len());
            for (idx, value) in values.into_iter().enumerate() {
                memory.write_byte(idx, value);
                assert_eq!(memory.read_byte(idx), value, "Error reading byte at {}", idx);
            }
        }

        #[test]
        fn memory_rw_word() {
            let mut memory = SimpleMemory::new(DataEndianness::Le);
            let values = [1u32, 2u32, 3u32, 4u32];
            memory.reserve_words(values.len());
            for (idx, value) in values.into_iter().enumerate() {
                memory.write_word(idx*4, value);
                assert_eq!(memory.read_word(idx*4, DataEndianness::Le), value, "Error reading byte at {}", idx);
            }
        }

        #[test]
        fn memory_rw_bytes_from_word() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let word = u32::from_be_bytes([0, 0, 0, 100u8]); //100
            memory.reserve_words(1);
            memory.write_word(0, word);
            assert_eq!(memory.read_word(0, DataEndianness::Be), word);
        }

        #[test]
        fn memory_rw_bytes() {
            let mut memory = SimpleMemory::new(DataEndianness::Be);
            let values = [1u8, 2u8, 3u8, 4u8];
            memory.reserve_bytes(values.len());
            memory.write_bytes(0, &values.to_vec());
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
            m.decode();
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
            m.decode();
            m.decode();
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
                m.decode();
            }
            m
        }

        fn isa_rvi32_mach(code: &str) -> (SimpleMachine, AssemblerTools) {
            let tools = build_code_repr(code);
            let mut m = new_machine_from_tools(&tools);
            let text = tools.text_section_words();
            let text_len = text.len();
            for _ in 0..text_len {
                m.decode();
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
            assert!(m.assert_pc(16));
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
                var1: .byte 0x4
                var2: .byte 0xa

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
            let start_addr = data_section.address + 4;
            let expected = vec![100u8];
            println!("{:?}", m.bytes());
            assert!(m.assert_memory_bytes(start_addr+1, expected.len(), &expected, 1));
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
                    lb t2, 0(t1)
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
            let (m, _) = isa_rvi32_mach(code);
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
            let (m, _) = isa_rvi32_mach(code);
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
            let (m, _) = isa_rvi32_mach(code);
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
            let (m, _) = isa_rvi32_mach(code);
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
            let (m, _) = isa_rvi32_mach(code);
            let t1 = Register::T1.id() as usize;
            let t2 = Register::T2.id() as usize;
            let regs = m.read_registers();
            assert_eq!(regs[t2], 0);
            assert_eq!(regs[t1], 8);
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
            writer.set_section_data(SectionName::Text, bytes, 4)
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
            writer.set_section_data(SectionName::Text, bytes, 4)
                .expect("error setting text data");
            let write_res = writer.save(filename);
            assert!(write_res.is_ok());

            // Reading temporary ELF file data
            let read_io_res = std::fs::read(filename);
            if let Ok(data) = read_io_res {
                let read_res = ElfReader::new(&data, DataEndianness::Le);
                assert!(read_res.is_ok());
            }
            else {
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
            writer.set_section_data(SectionName::Text, bytes_written.clone(), 4)
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
                    let bytes_read = &reader.section(".text").data;
                    assert_eq!(bytes_read, &bytes_written);
                }
                else {
                    assert!(read_res.is_ok());
                }
            }
            else {
                assert!(read_io_res.is_ok());
            }
        }
    }
}
