//intel syntax
pub mod intel {
    use std::str::Chars;

    const SINGLE_TOKEN: [char; 7] = [
        ':',
        ',',
        '.',
        '(',
        ')',
        '+',
        '-',
    ];

    fn handle_comment(it: &mut Chars) -> Option<char> {
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

    fn handle_whitespace(it: &mut Chars) -> Option<char> {
        it.next()
    }

    fn handle_hexadecimal(it: &mut Chars, n: &mut String) -> Option<char> {
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

    fn handle_decimal(it: &mut Chars, n: &mut String) -> Option<char> {
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

    fn handle_number(it: &mut Chars, number: &mut String) -> Option<char> {
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

    fn handle_string(it: &mut Chars, s: &mut String) -> Option<char> {
        let mut opt = it.next();
        let mut escaped = false;
        while let Some(ch) = opt {
            if ch == '\\' {
                escaped = true;
                s.push(ch);
                opt = it.next();
            }
            else if ch == '"' {
                if escaped {
                    escaped = false;
                    s.push(ch);
                    opt = it.next();
                }
                else {
                    s.push(ch);
                    opt = it.next();
                    break;
                }
            }
            else {
                s.push(ch);
                opt = it.next();
            }
        }
        opt
    }

    fn handle_name(it: &mut Chars, name: &mut String) -> Option<char> {
        let mut opt = it.next();
        while let Some(ch) = opt {
            if !ch.is_ascii_alphanumeric() {
                break;
            }
            name.push(ch);
            opt = it.next();
        }
        opt
    }

    pub fn get_tokens(buffer: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut it = buffer.chars();

        // TODO: string, hex numbers

        let mut opt = it.next();
        while let Some(ch) = opt {
            if SINGLE_TOKEN.contains(&ch) {
                tokens.push(String::from(ch));
                opt = it.next();
            }
            else if ch == '/' {
                opt = handle_comment(&mut it);
            }
            else if ch == '"' {
                let mut s = String::from(ch);
                opt = handle_string(&mut it, &mut s);
                tokens.push(s);
            }
            else if ch.is_whitespace() {
                opt = handle_whitespace(&mut it);
            }
            else if ch.is_digit(10) {
                let mut number = String::from(ch);
                opt = handle_number(&mut it, &mut number);
                tokens.push(number);
            }
            else if ch.is_ascii_alphabetic(){
                let mut s = String::from(ch);
                opt = handle_name(&mut it, &mut s);
                tokens.push(s);
            }
            else {
                eprintln!("{}, What?", ch);
                opt = it.next();
            }
        }

        tokens
    }
}

//intel syntax
pub mod at8t {
}