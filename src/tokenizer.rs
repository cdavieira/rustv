pub trait Tokenizer {
    fn get_tokens(&mut self, buffer: &str) -> Vec<String> ;
}



/* The following code was written to ease the implementation of the 'Tokenizer' trait. */

use std::iter::Iterator;

pub enum CommonClass {
    COMMENT,
    NUMBER,
    STRING,
    IDENTIFIER,
    UNIT,
    IGNORE,
    AMBIGUOUS
}

/**
Any entity which implements the 'tokenizer::CommonClassifier' trait can then use the function 'tokenizer::get_tokens' as the backend for the implementation of 'Tokenizer::get_tokens'

'tokenizer::CommonClassifier': a tokenization strategy where N chars can be mapped to one of the categories/variants stored in enum 'CommonClass'. The implementor is responsible for the mapping

By default, 'Tokenizer' is implemented for any entity which implements 'CommonClassifier'

OBS: 'Tokenizer::get_tokens' can still be overriden to use more complex/useful iterators
*/
pub trait CommonClassifier {
    fn is_ambiguous(&self, ch: char) -> bool ;
    fn is_unit(&self, ch: char) -> bool;
    fn is_comment(&self, ch: char) -> bool;
    fn is_identifier(&self, ch: char) -> bool;
    fn is_string(&self, ch: char) -> bool {
        ch == '"'
    }
    fn is_ignore(&self, ch: char) -> bool {
        ch.is_whitespace()
    }
    //all numbers usually begin with digits 0 to 9 (ex: 0x1, 2, 0o4, 0b0101, ...)
    fn is_number(&self, ch: char) -> bool {
        ch.is_digit(10)
    }

    fn handle_ambiguous(&self, it: &mut impl Iterator<Item = char>, s: &mut String) -> Option<char>;
    fn handle_comment(&self, it: &mut impl Iterator<Item = char>) -> Option<char>;
    fn handle_identifier(&self, it: &mut impl Iterator<Item = char>, name: &mut String) -> Option<char>;
    fn handle_unit(&self, it: &mut impl Iterator<Item = char>) -> Option<char> {
        it.next()
    }
    fn handle_ignore(&self, it: &mut impl Iterator<Item = char>) -> Option<char> {
        it.next()
    }
    fn handle_number(&self, it: &mut impl Iterator<Item = char>, number: &mut String) -> Option<char> {
        let mut opt: Option<char>;
        if number == "0" {
            opt = it.next();
            if let Some(ch) = opt {
                match ch {
                    'x'|'X' => {
                        number.push(ch);
                        opt = handle_hexadecimal(it, number);
                    },
                    digit if ch.is_ascii_digit() => {
                        number.push(digit);
                        opt = handle_decimal(it, number);
                    },
                    _ => {
                        opt = it.next();
                    }
                }
            }
        }
        else {
            opt = handle_decimal(it, number);
        }
        opt
    }
    //TODO: test this
    fn handle_string(&self, it: &mut impl Iterator<Item = char>, s: &mut String) -> Option<char> {
        let mut opt = it.next();
        while let Some(ch) = opt {
            if ch == '\\' {
                s.push(ch);
                opt = it.next();
                if let Some(ch) = opt {
                    match ch {
                        '"' => {
                            s.push(ch);
                            opt = it.next();
                        },
                        _ => {

                        }
                    }
                }
            }
            else if ch == '"' {
                s.push(ch);
                opt = it.next();
                break;
            }
            else {
                s.push(ch);
                opt = it.next();
            }
        }
        opt
    }
}

///Default implementation of 'Tokenizer' for any entity which implements 'CommonClassifier'
impl<T: CommonClassifier> Tokenizer for T {
    fn get_tokens(&mut self, buffer: &str) -> Vec<String>  {
        let mut it = buffer.chars();
        get_tokens(&mut it, self)
    }
}

fn get_tokens(
    it: &mut impl Iterator<Item = char>,
    classifier: &impl CommonClassifier
) -> Vec<String>
{
    let mut tokens = Vec::new();

    let mut opt = it.next();
    while let Some(ch) = opt {
        if classifier.is_ambiguous(ch) {
            let mut s = String::from(ch);
            opt = classifier.handle_ambiguous(it, &mut s);
            tokens.push(s);
        }
        else if classifier.is_unit(ch) {
            tokens.push(String::from(ch));
            opt = classifier.handle_unit(it);
        }
        else if classifier.is_comment(ch) {
            opt = classifier.handle_comment(it);
        }
        else if classifier.is_string(ch) {
            let mut s = String::from(ch);
            opt = classifier.handle_string(it, &mut s);
            tokens.push(s);
        }
        else if classifier.is_ignore(ch) {
            opt = classifier.handle_ignore(it);
        }
        else if classifier.is_number(ch) {
            let mut number = String::from(ch);
            opt = classifier.handle_number(it, &mut number);
            tokens.push(number);
        }
        else if classifier.is_identifier(ch){
            let mut s = String::from(ch);
            opt = classifier.handle_identifier(it, &mut s);
            tokens.push(s);
        }
        else {
            eprintln!("{}, What?", ch);
            opt = it.next();
        }
    }

    tokens
}

fn handle_number(it: &mut impl Iterator<Item = char>, n: &mut String, is_valid: impl Fn(char) -> bool) -> Option<char>{
        let mut opt = it.next();
        while let Some(ch) = opt {
            if !is_valid(ch) {
                break;
            }
            n.push(ch);
            opt = it.next();
        }
        opt
}

fn handle_hexadecimal(it: &mut impl Iterator<Item = char>, n: &mut String) -> Option<char> {
    handle_number(it, n, |ch| ch.is_ascii_hexdigit())
}

fn handle_decimal(it: &mut impl Iterator<Item = char>, n: &mut String) -> Option<char> {
    handle_number(it, n, |ch| ch.is_digit(10))
}
