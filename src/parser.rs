use crate::spec::{Extension, ToArg, ArgValue};

pub trait Parser {
    type Token;
    type Statement;

    fn parse(&self, token: Vec<Self::Token>) -> Vec<Self::Statement> ;
}



/* The following code was written to ease the process of implementing the 'Parser' trait. */

pub enum Keyword {
    PSEUDO,
    INSTRUCTION(Box<dyn Extension>),
    SECTION(String),
    LABEL(String),
}

pub trait ToKeyword {
    type Token;
    fn to_keyword(&self, token: &Self::Token) -> Option<Keyword> ;
    fn is_keyword(&self, token: &Self::Token) -> bool {
        self.to_keyword(token).is_some()
    }
}

pub trait TranslatePseudo {
    type Token: Clone;
    fn translate_pseudo(&self, stat: &Vec<Self::Token>) -> Option<Vec<Vec<Self::Token>>>;
}

//TODO: improve this
fn group_tokens<T>(
    decoder: &impl ToKeyword<Token = T>,
    tokens: Vec<T>
) -> Vec<Vec<T>>
{
    let mut v = Vec::new();
    let mut args = Vec::new();
    let it = tokens.into_iter().rev();
    for token in it {
        if decoder.is_keyword(&token) {
            let mut i = vec![token];
            args.reverse();
            for arg in args.drain(..) {
                i.push(arg);
            }
            v.push(i);
        }
        else {
            args.push(token);
        }
    }
    v.reverse();
    v
}

fn process_pseudos<T>(
    translator: &impl TranslatePseudo<Token = T>,
    stats: Vec<Vec<T>>
) -> Vec<Vec<T>>
{
    let mut v = Vec::new();
    for stat in stats {
        if let Some(translated) = translator.translate_pseudo(&stat) {
            v.extend(translated);
        }
        else {
            v.push(stat);
        }
    }
    v
}

// TODO:
// fn process_labels() {
//
// }

fn specialize_tokens<T>(
    handler: &(impl ToArg<Token = T> + ToKeyword<Token = T>),
    stats: Vec<Vec<T>>
) -> Vec<(Box<dyn Extension>, Vec<ArgValue>)>
{
    let mut v = Vec::new();
    for mut stat in stats {
        let token_args = stat.split_off(1);
        let token_kw = stat.get(0).unwrap();
        match handler.to_keyword(token_kw) {
            Some(Keyword::PSEUDO) => {

            },
            Some(Keyword::INSTRUCTION(e)) => {
                let mut args = Vec::new();
                for token_arg in token_args {
                    if let Some(a) = handler.to_arg(token_arg) {
                        args.push(a);
                    }
                }
                v.push((e, args));
            },
            Some(Keyword::SECTION(_)) => {

            },
            Some(Keyword::LABEL(_)) => {

            },
            _ => {

            },
        };
    }
    v
}

pub fn parse<T>(
    handler: &(impl ToArg<Token = T> + ToKeyword<Token = T> + TranslatePseudo<Token = T>),
    tokens: Vec<T>
) -> Vec<(Box<dyn Extension>, Vec<ArgValue>)>
{
    let stats = group_tokens(handler, tokens);
    let stats = process_pseudos(handler, stats);
    specialize_tokens(handler, stats)
}
