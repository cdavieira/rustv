use crate::spec::{Extension, ToArg, Arg};

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

pub trait ToKeyword<'a> {
    type Token;
    fn to_keyword(&self, token: &'a Self::Token) -> Option<Keyword<'a>> ;
    fn is_keyword(&self, token: &'a Self::Token) -> bool {
        self.to_keyword(token).is_some()
    }
}

fn group_tokens<'a, T>(
    decoder: &'a impl ToKeyword<'a, Token = T>,
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
    handler: &(impl ToArg<Token = T> + ToKeyword<'a, Token = T>),
    stats: Vec<Vec<&'a T>>
) -> Vec<(&'a Box<dyn Extension>, Vec<Arg>)>
{
    let mut v = Vec::new();
    for stat in &stats {
        let token_kw = stat.get(0).unwrap();
        let token_args = &stat[1..];
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
            Some(Keyword::SECTION(_)) => todo!(),
            Some(Keyword::LABEL(_)) => todo!(),
            _ => todo!(),
        };
    }
    v
}

pub fn parse<'a, T>(
    handler: &'a (impl ToArg<Token = T> + ToKeyword<'a, Token = T>),
    tokens: &'a Vec<T>
) -> Vec<(&'a Box<dyn Extension>, Vec<Arg>)>
{
    let stats = group_tokens(handler, tokens);
    specialize_tokens(handler, stats)
}




// pub trait TranslatePseudo<'a> {
// // pub trait TranslatePseudo {
//     type Token;
//     fn translate_pseudo(&self, stat: &Vec<&Self::Token>, pseudo_stat: &'a mut Vec<Self::Token>) -> Option<Vec<Vec<&'a Self::Token>>>;
//     // fn translate_pseudo(&self, stat: &Vec<&Self::Token>) -> Option<Vec<Vec<&Self::Token>>>;
// }
//
// // pub fn process_pseudos<'a, T>(translator: &'a impl TranslatePseudo<Token = T>, stats: Vec<Vec<&'a T>>) -> Vec<Vec<&'a T>> {
// pub fn process_pseudos<'a, T>(
//     translator: &'a impl TranslatePseudo<'a, Token = T>,
//     stats: &'a Vec<Vec<&'a T>>,
//     pseudo_stats: &'a mut Vec<T>,
//     new_stats: &mut Vec<&'a Vec<&'a T>>,
// ) -> ()
// {
//     new_stats.clear();
//     for stat in stats  {
//         if let Some(translated) = translator.translate_pseudo(stat, pseudo_stats) {
//             new_stats.extend(&translated);
//         }
//         else {
//             new_stats.push(stat);
//         }
//     }
// }

