pub trait Tokenizer {
    fn get_tokens(&mut self, buffer: &str) -> Vec<String> ;
}



/* The following code was written to ease the implementation of the 'Tokenizer' trait. */

use crate::streamreader::{CharStreamReader, Position, StreamReader};

pub enum CommonClass {
    Comment,
    Number,
    String,
    Identifier,
    Unit,
    Ignore,
    Ambiguous,
}

/**
'Tokenizer' is implemented for any entity which implements the 'tokenizer::CommonClassifier' trait

'tokenizer::CommonClassifier': a tokenization strategy where N chars can be mapped to one of the categories/variants stored in enum 'CommonClass'. The implementor is responsible for the mapping
*/
pub trait CommonClassifier {
    fn is_ambiguous(&self, ch: char) -> bool;
    fn handle_ambiguous(&self, it: &mut CharStreamReader) -> Option<String>;


    fn is_unit(&self, ch: char) -> bool;
    fn handle_unit(&self, it: &mut CharStreamReader) -> Option<String> {
        it.read_and_advance().map(|c| c.to_string())
    }


    fn is_comment(&self, ch: char) -> bool;
    fn handle_comment(&self, it: &mut CharStreamReader) -> Option<String>;


    fn is_identifier(&self, ch: char) -> bool;
    fn handle_identifier(&self, it: &mut CharStreamReader) -> Option<String>;


    fn is_string(&self, ch: char) -> bool {
        ch == '"'
    }
    fn handle_string(&self, it: &mut CharStreamReader) -> Option<String> {
        let Some(start) = it.current_token() else {
            return None;
        };

        let mut s = String::new();

        if start == '"' {
            s.push(start);
        }

        while let Some(ch) = it.advance_and_read() {
            s.push(ch);
            if ch == '"' {
                it.advance();
                break;
            }
            if ch == '\\' {
                match it.advance_and_read() {
                    Some('"') => s.push('"'),
                    _ => { },
                }
            }
        }

        Some(s)
    }


    fn is_ignore(&self, ch: char) -> bool {
        ch.is_whitespace()
    }
    fn handle_ignore(&self, it: &mut CharStreamReader) -> Option<String> {
        it.advance();
        None
    }


    //all numbers usually begin with digits from 0 to 9 (ex: 0x1, 2, 0o4, 0b0101, ...)
    fn is_number(&self, ch: char) -> bool {
        ch.is_digit(10)
    }
    fn handle_number(&self, it: &mut CharStreamReader) -> Option<String> {
        let Some(first_digit) = it.current_token() else {
            return None;
        };

        let Some(second_digit) = it.next_token() else {
            return handle_decimal(it);
        };

        if first_digit == '0' {
            match second_digit {
                'x' | 'X' => handle_hexadecimal(it),
                _ => handle_decimal(it),
            }
        }
        else {
            handle_decimal(it)
        }
    }


    fn is_token(&self, ch: char) -> Option<CommonClass> {
        if self.is_ambiguous(ch) {
            return Some(CommonClass::Ambiguous);
        }
        if self.is_unit(ch) {
            return Some(CommonClass::Unit);
        }
        if self.is_comment(ch) {
            return Some(CommonClass::Comment);
        }
        if self.is_string(ch) {
            return Some(CommonClass::String);
        }
        if self.is_ignore(ch) {
            return Some(CommonClass::Ignore);
        }
        if self.is_number(ch) {
            return Some(CommonClass::Number);
        }
        if self.is_identifier(ch){
            return Some(CommonClass::Identifier);
        }
        None
    }
    fn handle_token(
        &mut self,
        it: &mut CharStreamReader,
    ) -> Result<Option<String>>
    {
        let Some(ch) = it.current_token() else {
            return Ok(None);
        };
        match self.is_token(ch) {
            Some(CommonClass::Ambiguous)  => Ok(self.handle_ambiguous(it)),
            Some(CommonClass::Unit)       => Ok(self.handle_unit(it)),
            Some(CommonClass::Comment)    => Ok(self.handle_comment(it)),
            Some(CommonClass::String)     => Ok(self.handle_string(it)),
            Some(CommonClass::Ignore)     => Ok(self.handle_ignore(it)),
            Some(CommonClass::Number)     => Ok(self.handle_number(it)),
            Some(CommonClass::Identifier) => Ok(self.handle_identifier(it)),
            None => {
                let pos = it.current_position().unwrap_or(Position::new(0, 0, 0));
                Err(TokenizerError::AutomataException(pos))
            }
        }
    }
}

fn handle_number(it: &mut CharStreamReader, is_valid: impl Fn(char) -> bool) -> Option<String>{
    let mut n = String::new();
    while let Some(ch) = it.current_token() {
        if !is_valid(ch) {
            break;
        }
        n.push(ch);
        it.advance();
    }
    Some(n)
}

fn handle_hexadecimal(it: &mut CharStreamReader) -> Option<String> {
    let prefix = String::from("0x");
    it.advance();
    it.advance();
    let suffix = handle_number(it, |ch| ch.is_ascii_hexdigit()).unwrap();
    Some(prefix + &suffix)
}

fn handle_decimal(it: &mut CharStreamReader) -> Option<String> {
    handle_number(it, |ch| ch.is_digit(10))
}




type Result<T> = std::result::Result<T, TokenizerError>;

#[derive(Debug)]
pub enum TokenizerError {
    AutomataException(Position),
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizerError::AutomataException(pos) => 
                write!(f, "Automata exception at line {} column {}", pos.row(), pos.col()),
        }
    }
}




///Default implementation of 'Tokenizer' for any entity which implements 'CommonClassifier'
impl<T: CommonClassifier> Tokenizer for T {
    fn get_tokens(&mut self, buffer: &str) -> Vec<String>  {
        let mut it = CharStreamReader::new(buffer.chars(), '\n');
        let mut tokens = Vec::new();
        while it.current_token().is_some() {
            match self.handle_token(&mut it) {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => { }
                Err(err) => panic!("{}", err),
            }
        }
        tokens
    }
}
