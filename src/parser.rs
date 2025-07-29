use crate::lexer;
use crate::spec::{Extension, ArgValue};

pub trait Parser {
    type Token;
    type Statement;

    fn parse(&self, token: Vec<Self::Token>) -> Vec<Self::Statement> ;
}



/* The following code was written to ease the process of implementing the 'Parser' trait. */

//2.1 MAKING GROUPS OUT OF THE STREAM OF TOKENS

fn group_tokens(
    tokens: Vec<lexer::Token>
) -> Vec<(lexer::Token, Vec<lexer::Token>)>
{
    let mut token_groups = Vec::new();
    let mut args = Vec::new();
    for token in tokens.into_iter().rev() {
        match &token {
            lexer::Token::OP(_)        |
            lexer::Token::PSEUDO(_)    |
            lexer::Token::DIRECTIVE(_) |
            lexer::Token::SECTION(_)   |
            lexer::Token::LABEL(_) =>  {
                token_groups.push((token, args.drain(..).rev().collect()));
            },

            lexer::Token::REG(_)   |
            lexer::Token::NAME(_)  |
            lexer::Token::STR(_)   |
            lexer::Token::NUMBER(_) => {
                args.push(token);
            },

            _ => { }
        }
    }
    token_groups.into_iter().rev().collect()
}

// 2.2 ARGS INTO COPIABLE TYPES RESOLUTION
    
fn map_args_to_values(
    tokens: Vec<(lexer::Token, Vec<lexer::Token>)>
) -> Vec<(lexer::Token, Vec<lexer::Value>)>
{
    let mut v = Vec::new();
    for (kw, args) in tokens {
        let values = args
            .into_iter()
            .filter_map(|arg| match arg {
                //TODO: To map (resolved) offset we have to know that the number has between ( )
                //TODO: lexer::Token::NAME(_) => todo!(),
                lexer::Token::REG(register) => Some(lexer::Value::REGISTER(register)),
                lexer::Token::LABEL(s) =>  Some(lexer::Value::LABEL(s)),
                lexer::Token::NUMBER(n) =>  Some(lexer::Value::NUMBER(n)),
                _ => None
            })
            .collect();
        v.push((kw, values));
    }
    v
}

// 2.3 PSEUDO INSTRUCTION TRANSLATION

fn expand_pseudos(
    stats: Vec<(lexer::Token, Vec<lexer::Value>)>
) -> Vec<(lexer::Token, Vec<lexer::Value>)>
{
    let mut v = Vec::new();
    for (kw, args) in stats {
        match &kw {
            lexer::Token::PSEUDO(p) => {
                let m = p.translate(args).into_iter().map(|(i, a)| {
                    (lexer::Token::OP(i), a)
                });
                v.extend(m);
            },
            _ => {
                v.push((kw, args));
            }
        }
    }
    v
}

// TODO: 2.4  DIRECTIVE EXPANSION

// TODO: 2.5 SECTION HANDLING

pub enum Section {
    DATA,
    TEXT,
    CUSTOM(String)
}

// 2.6 GENERATING ADDRESSES

fn gen_addresses(
    stats: Vec<(lexer::Token, Vec<lexer::Value>)>
) -> Vec<(usize, lexer::Token, Vec<lexer::Value>)>
{
    let mut v = Vec::new();
    for (idx, (kw, args)) in stats.into_iter().enumerate() {
        v.push((idx * 4, kw, args))
    }
    v
}

// 2.7  SYMBOLIC LABEL TO ADDRESS

use std::collections::HashMap;

fn resolve_labels(
    stats: &Vec<(usize, lexer::Token, Vec<lexer::Value>)>
) -> HashMap<String, usize>
{
    let mut v = HashMap::new();
    for stat in stats {
        match &stat.1 {
            lexer::Token::LABEL(s) => {
                v.insert(s.clone(), stat.0);
            },
            _ => {
            }
        }
    }
    v
}

fn replace_symbolic_labels(
    stats: Vec<(usize, lexer::Token, Vec<lexer::Value>)>,
    symbmap: HashMap<String, usize>
) -> Vec<(usize, lexer::Token, Vec<lexer::Value>)>
{
    let mut v = Vec::new();
    let mut new_args = Vec::new();
    for (addr, kw, args) in stats {
        for arg in args {
            match arg {
                lexer::Value::LABEL(s) => {
                    if let Some(v) = symbmap.get(&s) {
                        new_args.push(lexer::Value::OFFSET(*v));
                    }
                    else {
                        eprintln!("Couldnt find {} in the symb map", s);
                    }
                },
                _ => {
                    new_args.push(arg);
                }
            }
        }
        v.push((addr, kw, new_args.drain(..).collect()))
    }
    v
}


// 3

use std::rc::Rc;

fn specialize_tokens(
    stats: Vec<(usize, lexer::Token, Vec<lexer::Value>)>
) ->  Vec<(usize, lexer::Token, Vec<ArgValue>)>
{
    let mut v = Vec::new();
    for (addr, kw, args) in stats {
        let new_args = args.iter().filter_map(|arg| match *arg {
            lexer::Value::NUMBER(n) => Some(ArgValue::NUMBER(n)),
            lexer::Value::REGISTER(register) => Some(ArgValue::REG(register.id().into())),
            lexer::Value::OFFSET(o) => Some(ArgValue::NUMBER(o.try_into().unwrap())),
            lexer::Value::LABEL(_) => panic!(),
        }).collect();
        v.push((addr, kw, new_args));
    }
    v
}

pub fn parse(
    tokens: Vec<lexer::Token>
) ->  Vec<(usize, lexer::Token, Vec<ArgValue>)>
{
    let stats = group_tokens(tokens);
    let stats = map_args_to_values(stats);
    let stats = expand_pseudos(stats);
    let stats = gen_addresses(stats);
    let m =  resolve_labels(&stats);
    let stats = replace_symbolic_labels(stats, m);
    specialize_tokens(stats)
}
