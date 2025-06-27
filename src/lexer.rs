use regex::Regex;
use crate::spec::{extensions, Register};

//https://www.reddit.com/r/Compilers/comments/z6qe98/best_approach_for_writing_a_lexer/
//https://craftinginterpreters.com/scanning.html

#[derive(Debug)]
pub enum Token {
    OP(extensions::rv32i::Opcode),
    REG(Register),
    NAME(String),
    SECTION(String),
    LABEL(String),
    NUMBER(i32),
    RET,
    DEC,
    HEX,
    LPAR,
    RPAR,
    COMMA,
}

pub fn at8t_from_string(code: &str) -> Vec<Token> {
    let pattern_name    = Regex::new(r"[a-zA-Z_]+").unwrap();
    let pattern_section = Regex::new(r"\.[a-zA-Z_]+").unwrap();
    let pattern_label   = Regex::new(r"[a-zA-Z_]+:").unwrap();
    let pattern_number  = Regex::new(r"-?\d+").unwrap();
    let pattern_par     = Regex::new(r"\(.*\)").unwrap();
    let word_to_token   = |word: &str| -> Option<Token> {
        match word.to_lowercase().as_str() {
            "addi" => Some(Token::OP(extensions::rv32i::Opcode::ADDI)),
            "sw"   => Some(Token::OP(extensions::rv32i::Opcode::SW)),
            "lw"   => Some(Token::OP(extensions::rv32i::Opcode::LW)),
            "ret"  => Some(Token::RET),
            // symb if pattern_par.is_match(symb) => word_to_token(word),
            symb if pattern_number.is_match(symb)  => Some(Token::NUMBER(symb.parse().unwrap())),
            symb if pattern_section.is_match(symb) => Some(Token::SECTION(String::from(symb))),
            symb if pattern_label.is_match(symb)   => Some(Token::LABEL(String::from(symb))),
            symb if pattern_name.is_match(symb)    => Some(Token::NAME(String::from(symb))),
            _ => None,
        }
    };

    code.trim()
        .split(|ch| match ch {
            ' ' | '\n' | '\t' => true,
            ',' => true,
            _ => false,
        })
        .map(word_to_token)
        .filter(|tok| tok.is_some())
        .map(|tok| tok.unwrap())
        .collect()
}
