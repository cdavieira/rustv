pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;

#[cfg(test)]
mod tests {
    //intel syntax
    mod intel {
        use super::super::*;

        #[test]
        fn tokenize_ignore_comments(){
            let code = "
                //this is gonna be great\n

                //awesome
            ";
            let expected: Vec<String> = vec![];
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }
        #[test]
        fn tokenize_words(){
            let code = "abc paulista oloco";
            let expected: Vec<String> = code
                .split_whitespace()
                .map(|s| String::from(s)).collect();
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_commas_parenthesis(){
            let code = "a, b, c(d, e(f)g, h";
            let expected: Vec<String> = code.chars()
                .filter(|ch| !matches!(ch, ' ' | '\n') )
                .map(|ch| String::from(ch))
                .collect();
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_labels(){
            let code = "
                main:
                loop2:
            ";
            let expected: Vec<String> = vec![String::from("main:"), String::from("loop2:")];
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_sections(){
            let code = "
                .globl
                .text
            ";
            let expected: Vec<String> = vec![String::from(".globl"), String::from(".text")];
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }

        #[test]
        fn tokenize_numbers(){
            let code = "-1 +3 -66 1000";
            let expected: Vec<String> = code.split_whitespace().map(|s| String::from(s)).collect();
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
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
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }

        /* lui, auipc, addi, andi, ori, xori, add, sub, and, or, xor, sll, srl, sra, fence, slti, sltiu, slli, srli, srai, slt, sltu, lw, sw */
        #[test]
        #[ignore]
        fn tokenize_rv32i(){
            todo!();
        }

        #[test]
        fn tokenize_strings(){
            let code = "\"isso ai\"  \"esse \\\"cara\\\"\"";
            let expected = [
                "\"isso ai\"", "\"esse \\\"cara\\\"\"",
            ].map(|s| String::from(s));
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
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
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
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
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
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
            let tokenizer = syntax::intel::Tokenizer;
            let res: Vec<String> = tokenizer::get_tokens(&tokenizer, code);
            assert_eq!(res, expected);
        }
    }
}
