pub trait Lexer {
    type Token;
    fn parse(&self, tokens: Vec<String>) -> Vec<<Self as Lexer>::Token> ;
}



/* The following code was written to ease the process of implementing the 'Lexer' trait. */

pub enum TokenClass {
    NUMBER,
    STRING,
    SYMBOL,
    REGISTER,
    OPCODE,
    IDENTIFIER,
    SECTION,
    DIRECTIVE,
    LABEL,
    CUSTOM,
    IGNORE
}

/**
Any entity which implements the 'lexer::Classifier' trait can then use the function 'lexer::parse' as the backend for the implementation of 'Lexer::parse'

'lexer::Classifier': a strategy where N chars can be mapped to one of the categories/variants stored in enum 'TokenClass'. The implementor is responsible for the mapping
*/
pub trait Classifier {
    type Token;

    fn is_symbol(&self, token: &str) -> bool ;
    fn is_register(&self, token: &str) -> bool ;
    fn is_opcode(&self, token: &str) -> bool ;
    fn is_identifier(&self, token: &str) -> bool ;
    fn is_section(&self, token: &str) -> bool ;
    fn is_directive(&self, _: &str) -> bool ;
    fn is_custom(&self, token: &str) -> bool ;
    fn is_label(&self, token: &str) -> bool ;
    fn is_number(&self, token: &str) -> bool {
        token.parse::<i32>().is_ok()
    }
    fn is_string(&self, token: &str) -> bool {
        token.starts_with('"') && token.ends_with('"')
    }

    fn str_to_number(&self, token: &str) -> Option<Self::Token>;
    fn str_to_string(&self, token: &str) -> Option<Self::Token>;
    fn str_to_symbol(&self, token: &str) -> Option<Self::Token>;
    fn str_to_register(&self, token: &str) -> Option<Self::Token>;
    fn str_to_opcode(&self, token: &str) -> Option<Self::Token>;
    fn str_to_identifier(&self, token: &str) -> Option<Self::Token>;
    fn str_to_section(&self, token: &str) -> Option<Self::Token>;
    fn str_to_directive(&self, token: &str) -> Option<Self::Token>;
    fn str_to_custom(&self, token: &str) -> Option<Self::Token>;
    fn str_to_label(&self, token: &str) -> Option<Self::Token>;

    fn classify(&self, token: &str) -> TokenClass {
        if self.is_symbol(token) {
            return TokenClass::SYMBOL;
        }

        if self.is_register(token) {
            return TokenClass::REGISTER;
        }

        if self.is_opcode(token) {
            return TokenClass::OPCODE;
        }

        if self.is_section(token) {
            return TokenClass::SECTION;
        }

        if self.is_custom(token) {
            return TokenClass::CUSTOM;
        }

        if self.is_label(token) {
            return TokenClass::LABEL;
        }

        if self.is_number(token) {
            return TokenClass::NUMBER;
        }

        if self.is_string(token) {
            return TokenClass::STRING;
        }

        if self.is_directive(token) {
            return TokenClass::DIRECTIVE;
        }

        if self.is_identifier(token) {
            return TokenClass::IDENTIFIER;
        }

        TokenClass::IGNORE
    }

    fn str_to_token(&self, token: &str) -> Option<Self::Token> {
        match self.classify(token) {
            TokenClass::LABEL => {
                Some(self.str_to_label(token).expect("str2label"))
            },
            TokenClass::NUMBER => {
                Some(self.str_to_number(token).expect("str2number"))
            },
            TokenClass::SYMBOL => {
                Some(self.str_to_symbol(token).expect("str2symbol"))
            },
            TokenClass::SECTION => {
                Some(self.str_to_section(token).expect("str2section"))
            },
            TokenClass::DIRECTIVE => {
                Some(self.str_to_directive(token).expect("str2directive"))
            },
            TokenClass::CUSTOM => {
                Some(self.str_to_custom(token).expect("str2custom"))
            },
            TokenClass::OPCODE => {
                Some(self.str_to_opcode(token).expect("str2opcode"))
            },
            TokenClass::STRING => {
                Some(self.str_to_string(token).expect("str2string"))
            },
            TokenClass::IDENTIFIER => {
                Some(self.str_to_identifier(token).expect("str2id"))
            },
            TokenClass::REGISTER => {
                Some(self.str_to_register(token).expect("str2register"))
            },
            TokenClass::IGNORE => {
                None
            }
        }
    }
}

pub fn parse<T>(lexer: & impl Classifier<Token = T>, tokens: Vec<String>) -> Vec<T> {
    let mut lexemes = Vec::new();

    for token in tokens {
        if let Some(lex) = lexer.str_to_token(&token) {
            lexemes.push(lex);
        }
    }

    lexemes
}
