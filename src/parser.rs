pub trait Parser<'a> {
    type Token;
    type Statement;

    fn token_to_stat(&self, token: &Self::Token) ->  Option<Self::Statement>;

    fn fill_stat(&self,
        it: &mut impl Iterator<Item = &'a Self::Token>,
        s: &mut Self::Statement
    ) -> Option<&'a Self::Token> ;

    fn get_instructions(&self, token: &'a Vec<Self::Token>) -> Vec<Self::Statement> {
        let mut v = Vec::new();
        let mut it = token.iter();
        let mut opt = it.next();
        while let Some(token) = opt {
            if let Some(mut st) = self.token_to_stat(&token) {
                opt = self.fill_stat(&mut it, &mut st);
                v.push(st);
            }
            else {
                opt = it.next();
            }
        }
        v
    }
}
