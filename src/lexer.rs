pub trait Lexer {
    type Token;

    fn str_to_token(&self, token: &str) -> Self::Token;

    fn parse(&self, tokens: Vec<String>) -> Vec<Self::Token> {
        let mut lexemes = Vec::new();
        
        for token in tokens {
            lexemes.push(self.str_to_token(&token));
        }

        lexemes
    }
}