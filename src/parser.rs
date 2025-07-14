use crate::spec::{Extension, WhichArg, Arg};

pub trait Parser<'a> {
    type Token;
    type Statement;

    fn parse(&'a self, token: &'a Vec<Self::Token>) -> Vec<Self::Statement> ;
}



/* The following code was written to ease the process of implementing the 'Parser' trait. */

pub enum Keyword<'a> {
    PSEUDO,
    INSTRUCTION(&'a Box<dyn Extension>),
    SECTION(String),
    LABEL(String),
}

pub trait WhichKeyword<'a> {
    type Token;
    fn which_keyword(&self, token: &'a Self::Token) -> Option<Keyword<'a>> ;
    fn is_keyword(&self, token: &'a Self::Token) -> bool {
        self.which_keyword(token).is_some()
    }
}

fn group_tokens<'a, T>(
    decoder: &'a impl WhichKeyword<'a, Token = T>,
    tokens: &'a Vec<T>
) -> Vec<Vec<&'a T>>
{
    let mut v = Vec::new();
    let mut it = tokens.iter();
    let mut opt = it.next();
    while let Some(token) = opt {
        if decoder.is_keyword(&token) {
            let mut i = vec![token];
            opt = it.next();
            while let Some(token) = opt {
                if decoder.is_keyword(token) {
                    break;
                }
                i.push(token);
                opt = it.next();
            }
            v.push(i);
        }
    }
    v
}

fn specialize_tokens<'a, T>(
    handler: &(impl WhichArg<Token = T> + WhichKeyword<'a, Token = T>),
    stats: Vec<Vec<&'a T>>
) -> Vec<(&'a Box<dyn Extension>, Vec<Arg>)>
{
    let mut v = Vec::new();
    for stat in &stats {
        let token_kw = stat.get(0).unwrap();
        let token_args = &stat[1..];
        let elem = match handler.which_keyword(token_kw) {
            Some(Keyword::PSEUDO) => todo!(),
            Some(Keyword::INSTRUCTION(e)) => {
                let mut args = Vec::new();
                for token_arg in token_args {
                    if let Some(a) = handler.which_arg(token_arg) {
                        args.push(a);
                    }
                }
                (e, args)
            },
            Some(Keyword::SECTION(_)) => todo!(),
            Some(Keyword::LABEL(_)) => todo!(),
            _ => todo!(),
        };
        v.push(elem);
    }
    v
}

pub fn parse<'a, T>(
    handler: &'a (impl WhichArg<Token = T> + WhichKeyword<'a, Token = T>),
    tokens: &'a Vec<T>
) -> Vec<(&'a Box<dyn Extension>, Vec<Arg>)>
{
    let stats = group_tokens(handler, tokens);
    specialize_tokens(handler, stats)
}
