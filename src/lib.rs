pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod reader;
pub mod assembler;

#[cfg(test)]
mod tests {
    //gas syntax
    mod gas {
        use crate::tokenizer::Tokenizer;
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        use crate::assembler::Assembler;

        use super::super::*;

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

        fn encode_single_instruction(code: &str) -> u32 {
            let mut tokenizer = syntax::gas::Tokenizer;
            let lexer = syntax::gas::Lexer;
            let parser = syntax::gas::Parser;
            let assembler = syntax::gas::Assembler;
            let tokens = tokenizer.get_tokens(code);
            let lexemes = lexer.parse(tokens);
            let stats = parser.parse(lexemes);
            let res = assembler.to_words(stats);
            *res.get(0).unwrap()
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
        #[ignore]
        fn encode_jal() {
            // let code = "jal t4,0x900";
            let code = "jal t4,18";
            let expected: u32 = 0x00000eef;
            let res = encode_single_instruction(code);
            assert_eq!(res, expected, "LEFT: {res:x}, RIGHT: {expected:x}");
        }
    }
}
