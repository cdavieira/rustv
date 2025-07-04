pub trait Parser<'a> {
    type Token;
    type Instruction;

    fn to_instruction(&self, token: &Self::Token) ->  Option<Self::Instruction>;

    fn handle_instruction(&self,
        it: &mut impl Iterator<Item = &'a Self::Token>,
        s: &mut Self::Instruction
    ) -> Option<&'a Self::Token> ;

    fn read_instruction(
        &self,
        it: & mut impl Iterator<Item = &'a Self::Token>,
        v: &mut Vec<&'a Self::Token>
    ) -> Option<&'a Self::Token>
    {
        let mut opt = it.next();
        while let Some(tok) = opt {
            match self.to_instruction(&tok) {
                Some(_) => break,
                None => {
                    v.push(&tok);
                    opt = it.next();
                }
            }
        }
        opt
    }

    fn parse(&self, token: &'a Vec<Self::Token>) -> Vec<Self::Instruction> {
        let mut v = Vec::new();
        let mut it = token.iter();
        let mut opt = it.next();
        while let Some(token) = opt {
            if let Some(mut st) = self.to_instruction(&token) {
                opt = self.handle_instruction(&mut it, &mut st);
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
