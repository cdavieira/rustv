use crate::tokenizer;
use crate::lexer;
use crate::parser;
use crate::spec::{extensions, Register};

pub mod intel {
    use super::*;

    pub struct Tokenizer ;
    pub struct Lexer;
    pub struct Parser;



    /* Tokenizer */

    impl tokenizer::Tokenizer for Tokenizer {
        fn needs_lookahead(&self, ch: char) -> bool {
            ch == '+' || ch == '-'
        }

        fn is_unit(&self, ch: char) -> bool {
            matches!(ch, ',' | '(' |')')
        }

        fn is_comment(&self, ch: char) -> bool {
            ch == '/'
        }

        fn is_name(&self, ch: char) -> bool {
            // ch.is_ascii_alphabetic()
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

        //TODO: names like abc:ddd, a.d.:e are being recognized...
        fn handle_name(&self, it: &mut impl Iterator<Item = char>, name: &mut String) -> Option<char> {
            let mut opt = it.next();
            while let Some(ch) = opt {
                // if !ch.is_ascii_alphanumeric() {
                if ch != ':' && ch != '.' && !ch.is_ascii_alphanumeric() {
                    break;
                }
                name.push(ch);
                opt = it.next();
            }
            opt
        }

        fn handle_lookahead(&self, it: &mut impl Iterator<Item = char>, s: &mut String) -> Option<char> {
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

    //TODO: how to simplify the process of extending supported specifications?
    //should i perhaps use interfaces for that? if so, how?
    //Maybe i could implement methods into the enum to do that....
    #[derive(Debug)]
    pub enum Pseudo {
        LI,
    }

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

    //TODO: create interface for this?
    fn is_register(raw: &str) -> bool {
        let s = raw.trim().to_lowercase();
        let is_xreg = {
            let x = &s[0..1];
            let n = &s[1..];
            x == "x" && n.parse::<i32>().is_ok()
        };
        let is_pc = s == "pc";
        is_xreg || is_pc
    }

    fn str_to_register(s: &str) -> Option<Register> {
        match s.trim().to_lowercase().as_str() {
            "x0" => Some(Register::X0),
            "x1" => Some(Register::X1),
            "x2" => Some(Register::X2),
            "x3" => Some(Register::X3),
            "x4" => Some(Register::X4),
            "x5" => Some(Register::X5),
            "x6" => Some(Register::X6),
            "x7" => Some(Register::X7),
            "x8" => Some(Register::X8),
            "x9" => Some(Register::X9),
            "x10" => Some(Register::X10),
            "x11" => Some(Register::X11),
            "x12" => Some(Register::X12),
            "x13" => Some(Register::X13),
            "x14" => Some(Register::X14),
            "x15" => Some(Register::X15),
            "x16" => Some(Register::X16),
            "x17" => Some(Register::X17),
            "x18" => Some(Register::X18),
            "x19" => Some(Register::X19),
            "x20" => Some(Register::X20),
            "x21" => Some(Register::X21),
            "x22" => Some(Register::X22),
            "x23" => Some(Register::X23),
            "x24" => Some(Register::X24),
            "x25" => Some(Register::X25),
            "x26" => Some(Register::X26),
            "x27" => Some(Register::X27),
            "x28" => Some(Register::X28),
            "x29" => Some(Register::X29),
            "x30" => Some(Register::X30),
            "x31" => Some(Register::X31),
            "pc" => Some(Register::PC),
            _ => None
        }
    }

    impl lexer::Lexer for Lexer {
        type Token = Token;

        fn str_to_token(&self, token: &str) -> Token {
            match token.to_lowercase().as_str() {
                "addi" => Token::OP(Opcode::RV32I(extensions::rv32i::Opcode::ADDI)),
                "sw" => Token::OP(Opcode::RV32I(extensions::rv32i::Opcode::SW)),
                "lw" => Token::OP(Opcode::RV32I(extensions::rv32i::Opcode::LW)),
                "li" => Token::PSEUDO(Pseudo::LI),
                "ret" => Token::RET,
                "," => Token::COMMA,
                "(" => Token::LPAR,
                ")" => Token::RPAR,
                "+" => Token::PLUS,
                "-" => Token::MINUS,
                _ => {
                    if let Ok(n) = token.parse::<i32>() {
                        Token::NUMBER(n)
                    } else if token.starts_with('.') {
                        Token::SECTION(token[1..].to_string())
                    } else if token.ends_with(':') {
                        Token::LABEL(token.trim_end_matches(':').to_string())
                    } else if token.starts_with('"') && token.ends_with('"') {
                        Token::STR(token.trim_matches('"').to_string())
                    } else if is_register(token) {
                        Token::REG(str_to_register(token).unwrap())
                    } else {
                        Token::NAME(token.to_string())
                    }
                }
            }
        }
    }



    /* Parser */

    #[derive(Debug)]
    pub enum Statement {
        Instruction{opcode: Opcode, args: Vec<Token>},
        Directive(String),
        Label(String),
    }

    impl<'a> parser::Parser<'a> for Parser {
        type Token = Token;
    
        type Statement = Statement;

        fn token_to_stat(&self, token: &Self::Token) -> Option<Self::Statement> {
            match token {
                Token::OP(opcode) => {
                    Some(Statement::Instruction {
                        opcode: *opcode,
                        args: vec![]
                    })
                }
                Token::LABEL(l) => Some(Statement::Label(String::from(l))),
                Token::SECTION(s) => Some(Statement::Directive(String::from(s))),
                _ => None
            }
        }
        
        fn fill_stat(&self,
            it: &mut impl Iterator<Item = &'a Self::Token>,
            s: &mut Self::Statement
        ) -> Option<&'a Self::Token> {
            match s {
                Statement::Instruction{opcode, args} => {
                    match opcode {
                        Opcode::RV32I(extensions::rv32i::Opcode::ADD) => {
                            //first param: read until field separator (comma)
                            //second param: read until 'token_to_stat' returns something other than None
                            todo!();
                        },
                        _ => {

                        }
                    }
                    it.next()
                },
                Statement::Directive(d) => {
                    it.next()

                },
                Statement::Label(l) => {
                    it.next()
                },
            }
        }
    }
}

mod att {

}
