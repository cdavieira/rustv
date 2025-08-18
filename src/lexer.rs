pub trait Lexer {
    type Token;
    fn parse(&self, tokens: Vec<String>) -> Vec<<Self as Lexer>::Token> ;
}



// Note: the code above was written to ease the process of implementing the 'Lexer' trait.



// Implementation 1

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
Any entity which implements the 'lexer::TokenClassifier' trait can then use the function 'lexer::parse' as the backend for the implementation of 'Lexer::parse'

'lexer::TokenClassifier': a strategy where N chars can be mapped to one of the categories/variants stored in enum 'TokenClass'. The implementor is responsible for the mapping
*/
pub trait TokenClassifier {
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
        let is_decimal = token.parse::<i32>().is_ok();
        let is_hex = if token.len() > 2 {
            let without_pref = &token[2..];
            i32::from_str_radix(without_pref, 16).is_ok()
        }
        else {
            false
        };
        is_decimal || is_hex
    }
    fn is_string(&self, token: &str) -> bool {
        token.starts_with('"') && token.ends_with('"')
    }

    fn str_to_number(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_string(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_symbol(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_register(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_opcode(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_identifier(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_section(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_directive(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_custom(&self, token: &str) -> Option<Self::Token> ;
    fn str_to_label(&self, token: &str) -> Option<Self::Token> ;

    fn classify(&self, token: &str) -> TokenClass {
        if self.is_symbol(token) {
            // println!("symbol!");
            return TokenClass::SYMBOL;
        }

        if self.is_register(token) {
            // println!("register!");
            return TokenClass::REGISTER;
        }

        if self.is_opcode(token) {
            // println!("opcode!");
            return TokenClass::OPCODE;
        }

        if self.is_directive(token) {
            // println!("directive!");
            return TokenClass::DIRECTIVE;
        }

        if self.is_section(token) {
            // println!("section!");
            return TokenClass::SECTION;
        }

        if self.is_custom(token) {
            // println!("custom!");
            return TokenClass::CUSTOM;
        }

        if self.is_label(token) {
            // println!("label!");
            return TokenClass::LABEL;
        }

        if self.is_number(token) {
            // println!("number!");
            return TokenClass::NUMBER;
        }

        if self.is_string(token) {
            // println!("string!");
            return TokenClass::STRING;
        }

        if self.is_identifier(token) {
            // println!("identifier!");
            return TokenClass::IDENTIFIER;
        }

        // println!("Ingore!");
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

fn parse<T>(lexer: & impl TokenClassifier<Token = T>, tokens: Vec<String>) -> Vec<T> {
    let mut lexemes = Vec::new();

    for token in tokens {
        // println!("Lexer is parsing {}...", token);
        if let Some(lex) = lexer.str_to_token(&token) {
            lexemes.push(lex);
        }
    }

    lexemes
}

impl<E, T: TokenClassifier<Token = E>> Lexer for T {
    type Token = E;

    fn parse(&self, tokens: Vec<String>) -> Vec<<Self as Lexer>::Token>  {
        parse(self, tokens)
    }
}





// Implementation 2

/*
'Implementation 1' is complemented by 'Implementation 2', which takes the responsability
of defining what a token is, instead of letting this detail to be implemented by the trait
implementor. This eases the process of building the parser later on, as the parser can reliably
work with that token definition, instead of having to work with a myriad of possible inputs.

In this implementation, it's the lexer job to:
0. (Optional, not recommended) Implement extensions to be supported as an Enum which implements the
   'spec::Extension' trait
1. Map the symbolic representation of all or a subset of the instructions of an Extension to their
   correspondent enum variant through the 'ToExtension' trait

2. (Optional) Implement the pseudo instructions to be supported as an Enum which
   implements the 'spec::Pseudo' trait
3. Map the symbolic representation of each pseudo instruction to their correspondent enum variant
   through the 'ToPseudo' trait

4. (Optional) Implement the directives to be supported as an Enum which implements the
   'spec::Directive' trait
5. Map the symbolic representation of each directive to their correspondent enum variant through
   the 'ToDirective' trait

7. Map the symbolic representation of registers to their correspondent enum variant through the
   'ToRegister' trait

OBS 1: Default implementation of extensions should be provided by this crate, as to standardize
how operations turn into bytes according to the RISCV specification

OBS 2: Default implementation of common pseudoinstructions/directives are provided by this crate,
as to standardize how operations turn into bytes according to the RISCV specification

=== About 'To*' Traits
'To*' traits allow implementers to link arbitrary data to Extensions, Pseudoinstrucions or
Directives

If the 'to_*' method returns the 'Some' variant, then 'token' maps to an existing
Extension/PseudoInstruction/Directive (thus grating support for that functionality)

On the other hand, if the return is the 'None' variant, then 'token' doesn't support any existing
Extension/PseudoInstruction/Directive
*/

// Token standardization

use crate::spec::{Extension, Pseudo, Directive, Register};

#[derive(Debug)]
pub enum Token {
    OP(Box<dyn Extension>),
    PSEUDO(Box<dyn Pseudo>),
    DIRECTIVE(Box<dyn Directive>),
    REG(Register),
    NAME(String),
    STR(String),
    LABEL(String),
    NUMBER(i32),
    SECTION,
    PLUS,
    MINUS,
    LPAR,
    RPAR,
    COMMA,
}

pub trait ToExtension<T> {
    fn to_extension(&self, token: T) -> Option<Box<dyn Extension>> ;

    fn in_extension(&self, token: T) -> bool {
        self.to_extension(token).is_some()
    }
}

pub trait ToPseudo {
    fn to_pseudo(&self, token: &str) -> Option<Box<dyn Pseudo>> ;

    fn is_pseudo(&self, token: &str) -> bool {
        self.to_pseudo(token).is_some()
    }
}

pub trait ToDirective {
    fn to_directive(&self, token: &str) -> Option<Box<dyn Directive>> ;

    fn is_directive(&self, token: &str) -> bool {
        self.to_directive(token).is_some()
    }
}

pub trait ToRegister {
    fn to_register(&self, token: &str) -> Option<Register> ;

    fn is_register(&self, token: &str) -> bool {
        self.to_register(token).is_some()
    }
}
