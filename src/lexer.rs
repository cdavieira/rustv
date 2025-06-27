use crate::spec::{extensions, Register};
use crate::tokenizer::{self, Tokenizer};

pub trait Lexer {
    type Token;

    fn str_to_token(&self, token: &str) -> Token;

    fn parse(&self, tokenizer: &impl Tokenizer, code: &str) -> Vec<Token> {
        let mut lexemes = Vec::new();
        
        let tokens: Vec<String> = tokenizer::get_tokens(tokenizer, code);
        for token in &tokens {
            lexemes.push(self.str_to_token(token));
        }

        lexemes
    }
}

#[derive(Debug)]
pub enum Token {
    OP(extensions::rv32i::Opcode),
    REG(Register),
    NAME(String),
    SECTION(String),
    LABEL(String),
    NUMBER(i32),
    RET,
    LPAR,
    RPAR,
    COMMA,
}

pub struct IntelLexer;

impl IntelLexer {
    fn is_register(s: &str) -> bool {
        let is_xreg = {
            let reg = s.trim().to_lowercase();
            let x = &reg[0..1];
            let n = &reg[1..];
            x.to_lowercase() == "x" && n.parse::<i32>().is_ok()
        };
        let is_pc = s.to_lowercase() == "pc";
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
}

impl Lexer for IntelLexer {
    type Token = Token;

    fn str_to_token(&self, token: &str) -> Token {
        match token.to_lowercase().as_str() {
            "addi" => Token::OP(extensions::rv32i::Opcode::ADDI),
            "sw" => Token::OP(extensions::rv32i::Opcode::SW),
            "lw" => Token::OP(extensions::rv32i::Opcode::LW),
            "ret" => Token::RET,
            "," => Token::COMMA,
            "(" => Token::LPAR,
            ")" => Token::RPAR,
            _ => {
                if let Ok(n) = token.parse::<i32>() {
                    Token::NUMBER(n)
                } else if token.starts_with('.') {
                    Token::SECTION(token[1..].to_string())
                } else if token.ends_with(':') {
                    Token::LABEL(token.trim_end_matches(':').to_string())
                } else if token.starts_with('"') && token.ends_with('"') {
                    Token::NAME(token.trim_matches('"').to_string())
                } else if Self::is_register(token) {
                    Token::REG(Self::str_to_register(token).unwrap())
                } else {
                    Token::NAME(token.to_string())
                }
            }
        }
    }
}
