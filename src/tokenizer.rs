use std::iter::Iterator;

pub trait Tokenizer {
    fn needs_lookahead(&self, ch: char) -> bool ;

    //single characters which are tokens by themselves (ex: '(', ')', ...)
    fn is_unit(&self, ch: char) -> bool;

    fn is_comment(&self, ch: char) -> bool;

    fn is_name(&self, ch: char) -> bool;

    fn is_whitespace(&self, ch: char) -> bool {
        ch.is_whitespace()
    }

    fn is_hexadecimal(&self, ch: char) -> bool {
        ch.is_digit(16)
    }

    fn is_decimal(&self, ch: char) -> bool {
        ch.is_digit(10)
    }

    fn is_string(&self, ch: char) -> bool {
        ch == '"'
    }

    fn is_number(&self, ch: char) -> bool {
        self.is_decimal(ch) || self.is_hexadecimal(ch)
    }

    
    fn handle_comment(&self, it: &mut impl Iterator<Item = char>) -> Option<char>;

    fn handle_name(&self, it: &mut impl Iterator<Item = char>, name: &mut String) -> Option<char>;

    fn handle_lookahead(&self, it: &mut impl Iterator<Item = char>, s: &mut String) -> Option<char>;

    fn handle_unit(&self, it: &mut impl Iterator<Item = char>) -> Option<char> {
        it.next()
    }

    fn handle_whitespace(&self, it: &mut impl Iterator<Item = char>) -> Option<char> {
        it.next()
    }

    fn handle_hexadecimal(&self, it: &mut impl Iterator<Item = char>, n: &mut String) -> Option<char> {
        let mut opt = it.next();
        while let Some(ch) = opt {
            if !ch.is_ascii_hexdigit() {
                break;
            }
            n.push(ch);
            opt = it.next();
        }
        opt
    }

    fn handle_decimal(&self, it: &mut impl Iterator<Item = char>, n: &mut String) -> Option<char> {
        let mut opt = it.next();
        while let Some(ch) = opt {
            if !ch.is_digit(10) {
                break;
            }
            n.push(ch);
            opt = it.next();
        }
        opt
    }

    fn handle_number(&self, it: &mut impl Iterator<Item = char>, number: &mut String) -> Option<char> {
        let mut opt: Option<char>;
        if number == "0" {
            opt = it.next();
            if let Some(ch) = opt {
                match ch {
                    'x'|'X' => {
                        number.push(ch);
                        opt = self.handle_hexadecimal(it, number);
                    },
                    digit if ch.is_ascii_digit() => {
                        number.push(digit);
                        opt = self.handle_decimal(it, number);
                    },
                    _ => {
                        opt = it.next();
                    }
                }
            }
        }
        else {
            opt = self.handle_decimal(it, number);
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

pub fn get_tokens(tokenizer: &impl Tokenizer, buffer: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut it = buffer.chars();

    let mut opt = it.next();
    while let Some(ch) = opt {
        if tokenizer.needs_lookahead(ch) {
            let mut s = String::from(ch);
            opt = tokenizer.handle_lookahead(&mut it, &mut s);
            tokens.push(s);
        }
        else if tokenizer.is_unit(ch) {
            tokens.push(String::from(ch));
            opt = tokenizer.handle_unit(&mut it);
        }
        else if tokenizer.is_comment(ch) {
            opt = tokenizer.handle_comment(&mut it);
        }
        else if tokenizer.is_string(ch) {
            let mut s = String::from(ch);
            opt = tokenizer.handle_string(&mut it, &mut s);
            tokens.push(s);
        }
        else if tokenizer.is_whitespace(ch) {
            opt = tokenizer.handle_whitespace(&mut it);
        }
        else if tokenizer.is_decimal(ch) {
            let mut number = String::from(ch);
            opt = tokenizer.handle_number(&mut it, &mut number);
            tokens.push(number);
        }
        else if tokenizer.is_name(ch){
            let mut s = String::from(ch);
            opt = tokenizer.handle_name(&mut it, &mut s);
            tokens.push(s);
        }
        else {
            eprintln!("{}, What?", ch);
            opt = it.next();
        }
    }

    tokens
}