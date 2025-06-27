pub mod lexer;
pub mod tokenizer;
pub mod spec;

#[cfg(test)]
mod tests {

    //syntax
    mod intel {
        use super::super::*;

        //extension
        mod rv32i {
            /* lui, auipc, addi, andi, ori, xori, add, sub, and, or, xor, sll, srl, sra, fence, slti, sltiu, slli, srli, srai, slt, sltu, lw, sw */
            
            #[test]
            fn get_tokens_0() {
                let code = "
                    loop:   beq  x11, x0,  exit
                            add  x11, x5,  x0
                            beq  x0,  x0,  loop
                ";

                let expected = vec![
                    String::from("loop"),
                    String::from(":"),
                    String::from("beq"),
                    String::from("x11"),
                    String::from(","),
                    String::from("x0"),
                    String::from(","),
                    String::from("exit"),
                    String::from("add"),
                    String::from("x11"),
                    String::from(","),
                    String::from("x5"),
                    String::from(","),
                    String::from("x0"),
                    String::from("beq"),
                    String::from("x0"),
                    String::from(","),
                    String::from("x0"),
                    String::from(","),
                    String::from("loop"),
                ];

                let v = super::tokenizer::intel::get_tokens(code);

                assert_eq!(v, expected);
            }

            #[test]
            fn get_tokens_1() {
                let code = "
                        .text
                        .globl main
                    main:
                        addi sp, sp, -16
                        sw   ra, 12(sp)
                        sw   s0, 8(sp)
                        addi s0, sp, 16
                ";

                let expected = vec![
                    String::from("."),
                    String::from("text"),
                    String::from("."),
                    String::from("globl"),
                    String::from("main"),
                    String::from("main"),
                    String::from(":"),
                    String::from("addi"),
                    String::from("sp"),
                    String::from(","),
                    String::from("sp"),
                    String::from(","),
                    String::from("-"),
                    String::from("16"),
                    String::from("sw"),
                    String::from("ra"),
                    String::from(","),
                    String::from("12"),
                    String::from("("),
                    String::from("sp"),
                    String::from(")"),
                    String::from("sw"),
                    String::from("s0"),
                    String::from(","),
                    String::from("8"),
                    String::from("("),
                    String::from("sp"),
                    String::from(")"),
                    String::from("addi"),
                    String::from("s0"),
                    String::from(","),
                    String::from("sp"),
                    String::from(","),
                    String::from("16"),
                ];

                let v = super::tokenizer::intel::get_tokens(code);

                assert_eq!(v, expected);
            }

            #[test]
            fn get_tokens_2() {
                let code = "
                        .text
                        .globl main
                    //this is gonna be great\n
                    main:
                        li   a0, 0

                        lw   ra, 12(sp)
                        lw   s0, 8(sp)
                        addi sp, sp, 16
                        ret
                ";

                let expected = vec![
                    String::from("."),
                    String::from("text"),
                    String::from("."),
                    String::from("globl"),
                    String::from("main"),
                    String::from("main"),
                    String::from(":"),
                    String::from("li"),
                    String::from("a0"),
                    String::from(","),
                    String::from("0"),
                    String::from("lw"),
                    String::from("ra"),
                    String::from(","),
                    String::from("12"),
                    String::from("("),
                    String::from("sp"),
                    String::from(")"),
                    String::from("lw"),
                    String::from("s0"),
                    String::from(","),
                    String::from("8"),
                    String::from("("),
                    String::from("sp"),
                    String::from(")"),
                    String::from("addi"),
                    String::from("sp"),
                    String::from(","),
                    String::from("sp"),
                    String::from(","),
                    String::from("16"),
                    String::from("ret"),
                ];

                let v = super::tokenizer::intel::get_tokens(code);

                assert_eq!(v, expected);
            }

            #[test]
            fn get_tokens_3() {
                let code = "
                        .text
                        .globl main
                    //this is gonna be great\n
                    main:
                        li   a0, 0

                        lw   ra, 12(sp)
                        lw   s0, 8(sp)
                        addi sp, sp, 16
                        ret
                ";

                let expected = vec![
                    String::from("."),
                    String::from("text"),
                    String::from("."),
                    String::from("globl"),
                    String::from("main"),
                    String::from("main"),
                    String::from(":"),
                    String::from("li"),
                    String::from("a0"),
                    String::from(","),
                    String::from("0"),
                    String::from("lw"),
                    String::from("ra"),
                    String::from(","),
                    String::from("12"),
                    String::from("("),
                    String::from("sp"),
                    String::from(")"),
                    String::from("lw"),
                    String::from("s0"),
                    String::from(","),
                    String::from("8"),
                    String::from("("),
                    String::from("sp"),
                    String::from(")"),
                    String::from("addi"),
                    String::from("sp"),
                    String::from(","),
                    String::from("sp"),
                    String::from(","),
                    String::from("16"),
                    String::from("ret"),
                ];

                let v = super::tokenizer::intel::get_tokens(code);

                assert_eq!(v, expected);
            }
        }
    }
}
