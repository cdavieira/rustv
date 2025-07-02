pub trait Parser<'a> {
    type Token;
    type Statement;

    fn token_to_stat(&self, token: &Self::Token) ->  Option<Self::Statement>;

    fn handle_stat(&self,
        it: &mut impl Iterator<Item = &'a Self::Token>,
        s: &mut Self::Statement
    ) -> Option<&'a Self::Token> ;

    fn read_instruction(
        &self,
        it: & mut impl Iterator<Item = &'a Self::Token>,
        v: &mut Vec<&'a Self::Token>
    ) -> Option<&'a Self::Token>
    {
        let mut opt = it.next();
        while let Some(tok) = opt {
            match self.token_to_stat(&tok) {
                Some(_) => break,
                None => {
                    v.push(&tok);
                    opt = it.next();
                }
            }
        }
        opt
    }

    fn get_instructions(&self, token: &'a Vec<Self::Token>) -> Vec<Self::Statement> {
        let mut v = Vec::new();
        let mut it = token.iter();
        let mut opt = it.next();
        while let Some(token) = opt {
            if let Some(mut st) = self.token_to_stat(&token) {
                opt = self.handle_stat(&mut it, &mut st);
                v.push(st);
            }
            else {
                opt = it.next();
            }
        }
        v
    }
}

// Generic driver to read instruction maybe?
// fn read_instruction_args<'a>(
//     parser: &impl parser::Parser<'a, Token = Token, Statement = Statement<'a>>,
//     it: & mut impl Iterator<Item = &'a Token>,
//     v: &mut Vec<&'a Token>
// ) -> Option<&'a Token>
// {
//     let mut opt = it.next();
//     while let Some(tok) = opt {
//         match parser.token_to_stat(&tok) {
//             Some(_) => break,
//             None => {
//                 v.push(&tok);
//                 opt = it.next();
//             }
//         }
//     }
//     opt
// }
