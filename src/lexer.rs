use crate::lang::highassembly::{ArgValue, KeyValue};
use crate::streamreader::{StreamReader, StringStreamReader};

#[derive(Debug)]
pub enum GenericToken {
    KeyToken(KeyValue),
    ArgToken(ArgValue),
}

pub trait ToGenericToken {
    fn to_generic_token(self) -> Option<GenericToken>;
}

pub trait Lexer {
    type Token: ToGenericToken;
    fn parse(&self, tokens: Vec<String>) -> Vec<<Self as Lexer>::Token> ;
}



// Note: the code below was written to ease the process of implementing the 'Lexer' trait.



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

    fn str_to_number(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_string(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_symbol(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_register(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_opcode(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_identifier(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_section(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_directive(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_custom(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;
    fn str_to_label(&self, it: &mut StringStreamReader) -> Option<Self::Token> ;

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

        if self.is_directive(token) {
            return TokenClass::DIRECTIVE;
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

        if self.is_identifier(token) {
            return TokenClass::IDENTIFIER;
        }

        TokenClass::IGNORE
    }

    fn str_to_token(&self, it: &mut StringStreamReader) -> Option<Self::Token> {
        let token = it.current_token().expect("Lexer failed when retrieving token");
        match self.classify(token.as_str()) {
            TokenClass::LABEL      => self.str_to_label(it),
            TokenClass::NUMBER     => self.str_to_number(it),
            TokenClass::SYMBOL     => self.str_to_symbol(it),
            TokenClass::SECTION    => self.str_to_section(it),
            TokenClass::DIRECTIVE  => self.str_to_directive(it),
            TokenClass::CUSTOM     => self.str_to_custom(it),
            TokenClass::OPCODE     => self.str_to_opcode(it),
            TokenClass::STRING     => self.str_to_string(it),
            TokenClass::IDENTIFIER => self.str_to_identifier(it),
            TokenClass::REGISTER   => self.str_to_register(it),
            TokenClass::IGNORE     => None
        }
    }
}

/*
The 'Lexer' Trait is implemented for any entity which implements the 'lexer::TokenClassifier' trait
*/
impl<T: ToGenericToken, C: TokenClassifier<Token = T>> Lexer for C {
    type Token = T;

    fn parse(&self, tokens: Vec<String>) -> Vec<<Self as Lexer>::Token>  {
        let mut lexemes = Vec::new();
        let mut it = StringStreamReader::new(tokens.into_iter(), String::from("\n"));
        while let Some(_) = it.current_token() {
            if let Some(lex) = self.str_to_token(&mut it) {
                lexemes.push(lex);
            }
            it.advance();
        }
        lexemes
    }
}





/*
In this implementation, it's the lexer job to:
0. (Optional, not recommended) Implement extensions to be supported as an Enum which implements the 'Extension' trait
1. Map the symbolic representation of all or a subset of the instructions of an Extension to their
   correspondent enum variant through the 'ToExtension' trait

2. (Optional) Implement pseudo instructions to be supported as an Enum which implements the 'Pseudo' trait
3. Map the symbolic representation of each pseudo instruction to their correspondent enum variant
   through the 'ToPseudo' trait

4. (Optional) Implement the directives to be supported as an Enum which implements the 'Directive' trait
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

use crate::lang::{
    ext::Extension,
    pseudo::Pseudo,
    directive::Directive,
    highassembly::Register,
};

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
