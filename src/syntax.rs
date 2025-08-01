pub mod gas {
    use crate::assembler::{self};
    use crate::tokenizer::CommonClassifier;
    use crate::lexer::{self, ToDirective, ToExtension, ToPseudo, ToRegister, TokenClassifier};
    use crate::parser::{self};
    use crate::spec::{self, Directive, DirectiveInstruction, Extension, Pseudo, PseudoInstruction, Register, RV32I };


    /* Tokenizer */

    pub struct Tokenizer ;

    impl CommonClassifier for Tokenizer {
        fn is_ambiguous(&self, ch: char) -> bool {
            ch == '+' || ch == '-'
        }

        fn is_unit(&self, ch: char) -> bool {
            matches!(ch, ',' | '(' |')')
        }

        fn is_comment(&self, ch: char) -> bool {
            ch == '/'
        }

        fn is_identifier(&self, ch: char) -> bool {
            ch.is_ascii_alphanumeric() || ch == '.' || ch == ':'
        }

        fn handle_comment(&self, it: &mut impl Iterator<Item = char>) -> Option<char> {
            let mut opt = it.next();
            while let Some(ch) = opt {
                if ch == '\n' {
                    opt = it.next();
                    break;
                }
                opt = it.next();
            }
            opt
        }

        fn handle_identifier(&self, it: &mut impl Iterator<Item = char>, name: &mut String) -> Option<char> {
            let mut opt = it.next();
            while let Some(ch) = opt {
                if !self.is_identifier(ch) {
                    break;
                }
                name.push(ch);
                opt = it.next();
            }
            opt
        }

        fn handle_ambiguous(&self, it: &mut impl Iterator<Item = char>, s: &mut String) -> Option<char> {
            let mut ch = it.next();
            while let Some(lookahead) = ch {
                if !self.is_number(lookahead) {
                    break;
                }
                s.push(lookahead);
                ch = it.next();
            }
            ch
        }
    }



    /* Lexer */

    pub struct Lexer ;

    impl ToRegister for Lexer {
        fn to_register(&self, token: &str) -> Option<crate::spec::Register>  {
            match token {
                "x0" | "zero" => Some(Register::X0),
                "x1" | "ra"   => Some(Register::X1),
                "x2" | "sp"   => Some(Register::X2),
                "x3" | "gp"   => Some(Register::X3),
                "x4" | "tp"   => Some(Register::X4),
                "x5" | "t0"   => Some(Register::X5),
                "x6" | "t1"   => Some(Register::X6),
                "x7" | "t2"   => Some(Register::X7),
                "x8" | "s0" | "fp" => Some(Register::X8),
                "x9" | "s1"   => Some(Register::X9),
                "x10" | "a0"  => Some(Register::X10),
                "x11" | "a1"  => Some(Register::X11),
                "x12" | "a2"  => Some(Register::X12),
                "x13" | "a3"  => Some(Register::X13),
                "x14" | "a4"  => Some(Register::X14),
                "x15" | "a5"  => Some(Register::X15),
                "x16" | "a6"  => Some(Register::X16),
                "x17" | "a7"  => Some(Register::X17),
                "x18" | "s2"  => Some(Register::X18),
                "x19" | "s3"  => Some(Register::X19),
                "x20" | "s4"  => Some(Register::X20),
                "x21" | "s5"  => Some(Register::X21),
                "x22" | "s6"  => Some(Register::X22),
                "x23" | "s7"  => Some(Register::X23),
                "x24" | "s8"  => Some(Register::X24),
                "x25" | "s9"  => Some(Register::X25),
                "x26" | "s10" => Some(Register::X26),
                "x27" | "s11" => Some(Register::X27),
                "x28" | "t3"  => Some(Register::X28),
                "x29" | "t4"  => Some(Register::X29),
                "x30" | "t5"  => Some(Register::X30),
                "x31" | "t6"  => Some(Register::X31),
                "pc" => Some(Register::PC),
                _ => None
            }
        }
    }

    impl ToPseudo for Lexer {
        fn to_pseudo(&self, token: &str) -> Option<Box<dyn Pseudo>>  {
            match token {
                "ret" => Some(Box::new(PseudoInstruction::RET)),
                "li"  => Some(Box::new(PseudoInstruction::LI)),
                "mv"  => Some(Box::new(PseudoInstruction::MV)),
                "la"  => Some(Box::new(PseudoInstruction::LA)),
                _ => None
            }
        }
    }

    impl ToDirective for Lexer {
        fn to_directive(&self, token: &str) -> Option<Box<dyn Directive>>  {
            match token {
                ".word" => Some(Box::new(DirectiveInstruction::WORD)),
                _ => None
            }
        }
    }

    impl ToExtension<&str> for Lexer {
        fn to_extension(&self, token: &str) -> Option<Box<dyn Extension>> {
            match token {
                "lui"   => Some(Box::new(RV32I::LUI))  ,
                "auipc" => Some(Box::new(RV32I::AUIPC)),
                "addi"  => Some(Box::new(RV32I::ADDI)) ,
                "andi"  => Some(Box::new(RV32I::ANDI)) ,
                "ori"   => Some(Box::new(RV32I::ORI))  ,
                "xori"  => Some(Box::new(RV32I::XORI)) ,
                "add"   => Some(Box::new(RV32I::ADD))  ,
                "sub"   => Some(Box::new(RV32I::SUB))  ,
                "and"   => Some(Box::new(RV32I::AND))  ,
                "or"    => Some(Box::new(RV32I::OR))   ,
                "xor"   => Some(Box::new(RV32I::XOR))  ,
                "sll"   => Some(Box::new(RV32I::SLL))  ,
                "srl"   => Some(Box::new(RV32I::SRL))  ,
                "sra"   => Some(Box::new(RV32I::SRA))  ,
                "fence" => Some(Box::new(RV32I::FENCE)),
                "slti"  => Some(Box::new(RV32I::SLTI)) ,
                "sltiu" => Some(Box::new(RV32I::SLTIU)),
                "slli"  => Some(Box::new(RV32I::SLLI)) ,
                "srli"  => Some(Box::new(RV32I::SRLI)) ,
                "srai"  => Some(Box::new(RV32I::SRAI)) ,
                "slt"   => Some(Box::new(RV32I::SLT))  ,
                "sltu"  => Some(Box::new(RV32I::SLTU)) ,
                "lw"    => Some(Box::new(RV32I::LW))   ,
                "lh"    => Some(Box::new(RV32I::LH))   ,
                "lhu"   => Some(Box::new(RV32I::LHU))  ,
                "lb"    => Some(Box::new(RV32I::LB))   ,
                "lbu"   => Some(Box::new(RV32I::LBU))  ,
                "sw"    => Some(Box::new(RV32I::SW))   ,
                "sh"    => Some(Box::new(RV32I::SH))   ,
                "sb"    => Some(Box::new(RV32I::SB))   ,
                "jal"   => Some(Box::new(RV32I::JAL))  ,
                "jalr"  => Some(Box::new(RV32I::JALR)) ,
                "ecall" => Some(Box::new(RV32I::ECALL)),
                "beq"   => Some(Box::new(RV32I::BEQ))  ,
                "bne"   => Some(Box::new(RV32I::BNE))  ,
                "blt"   => Some(Box::new(RV32I::BLT))  ,
                "bltu"  => Some(Box::new(RV32I::BLTU)) ,
                "bge"   => Some(Box::new(RV32I::BGE))  ,
                "bgeu"  => Some(Box::new(RV32I::BGEU)) ,
                _ => None,
            }
        }
    }

    impl TokenClassifier for Lexer {
        type Token = lexer::Token;

        fn is_register(&self, token: &str) -> bool {
            ToRegister::is_register(self, token)
        }

        fn is_symbol(&self, token: &str) -> bool{
            matches!(token, "," | "(" | ")" | "+" | "-")
        }

        fn is_opcode(&self, token: &str) -> bool {
            self.in_extension(token)
        }

        fn is_identifier(&self, token: &str) -> bool {
            let mut chs = token.chars();
            let f: char = chs.nth(0).unwrap_or(' ');
            f.is_ascii_alphabetic() && chs.all(|ch| ch.is_ascii_alphanumeric())
        }

        fn is_section(&self, token: &str) -> bool {
            token.starts_with('.')
        }

        fn is_directive(&self, token: &str) -> bool {
            ToDirective::is_directive(self, token)
        }

        fn is_label(&self, token: &str) -> bool {
            token.ends_with(':')
        }

        fn is_custom(&self, token: &str) -> bool {
            ToPseudo::is_pseudo(self, token)
        }

        fn str_to_number(&self, token: &str) -> Option<Self::Token> {
            let Ok(n) = token.parse::<i32>() else {
                return None;
            };
            Some(lexer::Token::NUMBER(n))
        }

        fn str_to_string(&self, token: &str) -> Option<Self::Token> {
            Some(lexer::Token::STR(token.trim_matches('"').to_string()))
        }

        fn str_to_symbol(&self, token: &str) -> Option<Self::Token> {
            match token {
                "," => Some(lexer::Token::COMMA),
                "(" => Some(lexer::Token::LPAR),
                ")" => Some(lexer::Token::RPAR),
                "+" => Some(lexer::Token::PLUS),
                "-" => Some(lexer::Token::MINUS),
                _ => None
            }
        }

        fn str_to_opcode(&self, token: &str) -> Option<Self::Token> {
            match self.to_extension(token) {
                Some(e) => {
                    Some(lexer::Token::OP(e))
                },
                None => None
            }
        }

        fn str_to_identifier(&self, token: &str) -> Option<Self::Token> {
            Some(lexer::Token::NAME(token.to_string()))
        }

        fn str_to_section(&self, token: &str) -> Option<Self::Token> {
            Some(lexer::Token::SECTION(token[1..].to_string()))
        }

        fn str_to_directive(&self, token: &str) -> Option<Self::Token> {
            if let Some(d) = ToDirective::to_directive(self, token) {
                Some(lexer::Token::DIRECTIVE(d))
            }
            else {
                None
            }
        }

        fn str_to_register(&self, token: &str) -> Option<Self::Token> {
            let Some(reg) = ToRegister::to_register(self, token.trim().to_lowercase().as_str()) else {
                return None;
            };
            Some(lexer::Token::REG(reg))
        }

        fn str_to_custom(&self, token: &str) -> Option<Self::Token> {
            if let Some(p) = ToPseudo::to_pseudo(self, token) {
                Some(lexer::Token::PSEUDO(p))
            }
            else {
                None
            }
        }

        fn str_to_label(&self, token: &str) -> Option<Self::Token> {
            Some(lexer::Token::LABEL(token.trim_end_matches(':').to_string()))
        }
    }



    /* Parser */

    pub struct Parser;

    impl parser::Parser for Parser {
        type Token = lexer::Token;
        type Statement = spec::AssemblyInstruction;

        fn parse(&self, tokens: Vec<Self::Token>) -> Vec<Self::Statement>  {
            parser::parse(tokens)
        }
    }



    /* Assembler */

    pub struct Assembler;

    impl assembler::Assembler for Assembler {
        type Instruction = spec::AssemblyInstruction;

        fn to_words(&self, instructions: Vec<Self::Instruction>) -> Vec<u32>  {
            assembler::to_u32(instructions)
        }
    }




    /* Loader */
}
