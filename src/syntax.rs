use crate::tokenizer::{self, CommonClassifier};
use crate::lexer::{self, TokenClassifier};
use crate::parser;
use crate::spec::{extensions, Register};
use crate::reader;

pub mod intel {
    use std::ops::Range;
    use super::*;

    pub struct Tokenizer ;
    pub struct Lexer;
    pub struct Parser;


    /* Tokenizer */

    impl tokenizer::CommonClassifier for Tokenizer {
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
            ch.is_ascii_alphabetic() || ch == '.' || ch == ':'
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
                if ch != ':' && ch != '.' && !ch.is_ascii_alphanumeric() {
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

    impl tokenizer::Tokenizer for Tokenizer {
        fn get_tokens(&mut self, buffer: &str) -> Vec<String> {
            // let mut it = reader::SimpleReader::new(buffer);
            let mut it = buffer.chars();
            tokenizer::get_tokens(&mut it, self)
        }
    }



    /* Lexer */
    #[derive(Debug, Copy, Clone)]
    pub enum Pseudo {
        LI,
    }

    //TODO: how to simplify the process of extending supported specifications? trait objects?
    #[derive(Debug, Copy, Clone)]
    pub enum Opcode {
        RV32I(extensions::rv32i::Opcode),
    }

    #[derive(Debug)]
    pub enum Token {
        OP(Opcode),
        PSEUDO(Pseudo),
        REG(Register),
        NAME(String),
        STR(String),
        SECTION(String),
        LABEL(String),
        NUMBER(i32),
        RET,
        PLUS,
        MINUS,
        LPAR,
        RPAR,
        COMMA,
    }

    fn register_matches(token: &str, prefix: &str, range: Range<i32>) -> bool {
        if let Ok(n) = token[1..].parse::<i32>() {
            &token[0..1] == prefix && range.contains(&n)
        }
        else {
            false
        }
    }

    const SYMBOLIC_REGISTERS: [&str; 7] = ["zero", "ra", "sp", "gp", "tp", "fp", "pc"];
    const OPCODE_CORE: [&str; 1] = ["ret"];
    const OPCODE_RV32I: [&str; 3] = ["sw", "addi", "lw"];
    const PSEUDO: [&str; 1] = ["li"];

    impl lexer::TokenClassifier for Lexer {
        type Token = Token;

        fn is_register(&self, token: &str) -> bool {
            let is_xreg = register_matches(token, "x", 0..32);
            let is_treg = register_matches(token, "t", 0..7);
            let is_areg = register_matches(token, "a", 0..8);
            let is_sreg = register_matches(token, "s", 0..12);
            let is_symbolic = SYMBOLIC_REGISTERS.contains(&token);
            is_xreg || is_treg || is_areg || is_sreg || is_symbolic
        }

        fn is_symbol(&self, token: &str) -> bool{
            matches!(token, "," | "(" | ")" | "+" | "-")
        }

        fn is_opcode(&self, token: &str) -> bool {
            OPCODE_CORE.contains(&token) || OPCODE_RV32I.contains(&token)
        }

        fn is_identifier(&self, token: &str) -> bool {
            let mut chs = token.chars();
            let f: char = chs.nth(0).unwrap_or(' ');
            f.is_ascii_alphabetic() && chs.all(|ch| ch.is_ascii_alphanumeric())
        }

        fn is_section(&self, token: &str) -> bool {
            token.starts_with('.')
        }

        fn is_directive(&self, _: &str) -> bool {
            false
        }

        fn is_label(&self, token: &str) -> bool {
            token.ends_with(':')
        }

        fn is_custom(&self, token: &str) -> bool {
            PSEUDO.contains(&token)
        }

        fn str_to_number(&self, token: &str) -> Option<Self::Token> {
            let Ok(n) = token.parse::<i32>() else {
                return None;
            };
            Some(Token::NUMBER(n))
        }

        fn str_to_string(&self, token: &str) -> Option<Self::Token> {
            Some(Token::STR(token.trim_matches('"').to_string()))
        }

        fn str_to_symbol(&self, token: &str) -> Option<Self::Token> {
            match token {
                "," => Some(Token::COMMA),
                "(" => Some(Token::LPAR),
                ")" => Some(Token::RPAR),
                "+" => Some(Token::PLUS),
                "-" => Some(Token::MINUS),
                _ => None
            }
        }

        fn str_to_opcode(&self, token: &str) -> Option<Self::Token> {
            match token {
                "ret" => Some(Token::RET),
                "addi" => Some(Token::OP(Opcode::RV32I(extensions::rv32i::Opcode::ADDI))),
                "sw"   => Some(Token::OP(Opcode::RV32I(extensions::rv32i::Opcode::SW))),
                "lw"   => Some(Token::OP(Opcode::RV32I(extensions::rv32i::Opcode::LW))),
                _ => None
            }
        }

        fn str_to_identifier(&self, token: &str) -> Option<Self::Token> {
            Some(Token::NAME(token.to_string()))
        }

        fn str_to_section(&self, token: &str) -> Option<Self::Token> {
            Some(Token::SECTION(token[1..].to_string()))
        }

        fn str_to_directive(&self, _: &str) -> Option<Self::Token> {
            None
        }

        fn str_to_register(&self, token: &str) -> Option<Self::Token> {
            let reg = match token.trim().to_lowercase().as_str() {
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
                "x28" | "t3"  => Some(Register::X27),
                "x29" | "t4"  => Some(Register::X27),
                "x30" | "t5"  => Some(Register::X30),
                "x31" | "t6"  => Some(Register::X31),
                "pc" => Some(Register::PC),
                _ => None
            };
            let Some(reg) = reg else {
                return None;
            };
            Some(Token::REG(reg))
        }

        fn str_to_custom(&self, token: &str) -> Option<Self::Token> {
            match token {
                "li" => Some(Token::PSEUDO(Pseudo::LI)),
                _ => None
            }
        }

        fn str_to_label(&self, token: &str) -> Option<Self::Token> {
            Some(Token::LABEL(token.trim_end_matches(':').to_string()))
        }
    }

    impl lexer::Lexer for Lexer {
        type Token = Token;

        fn parse(&self, tokens: Vec<String>) -> Vec<Token> {
            lexer::parse(self, tokens)
        }
    }



    /* Parser */

    #[derive(Debug)]
    pub enum Command {
        OP(Opcode),
        PSEUDO(Pseudo),
        RET
    }

    #[derive(Debug)]
    pub enum Statement<'a> {
        Instruction{opcode: Command, args: Vec<&'a Token>},
        Directive(String),
        Label(String),
    }

    impl<'a> parser::Parser<'a> for Parser {
        type Token = Token;
    
        type Instruction = Statement<'a>;

        fn to_instruction(&self, token: &Self::Token) -> Option<Self::Instruction> {
            match token {
                Token::OP(opcode) => {
                    Some(Statement::Instruction {
                        opcode: Command::OP(*opcode),
                        args: vec![]
                    })
                },
                Token::PSEUDO(pseudo) => {
                    Some(Statement::Instruction {
                        opcode: Command::PSEUDO(*pseudo),
                        args: vec![]
                    })
                },
                Token::RET => {
                    Some(Statement::Instruction {
                        opcode: Command::RET,
                        args: vec![]
                    })
                },
                Token::LABEL(l) => Some(Statement::Label(String::from(l))),
                Token::SECTION(s) => Some(Statement::Directive(String::from(s))),
                _ => None
            }
        }
        
        fn handle_instruction(&self,
            it: &mut impl Iterator<Item = &'a Self::Token>,
            s: &mut Self::Instruction
        ) -> Option<&'a Self::Token> {
            match s {
                Statement::Instruction{opcode: _, args} => {
                    self.read_instruction(it, args)
                },
                Statement::Directive(_) => {
                    it.next()
                },
                Statement::Label(_) => {
                    it.next()
                },
            }
        }
    }
}

mod att {

}
