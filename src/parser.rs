use crate::lexer;
use crate::spec::{ArgValue, AssemblyInstruction, AssemblySection, KeyValue};

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



// 2.2 ARGS INTO CLONEABLE TYPES RESOLUTION
    
fn map_args_to_values(
    tokens: Vec<(lexer::Token, Vec<lexer::Token>)>
) -> Vec<(lexer::Token, Vec<ArgValue>)>
{
    let mut v = Vec::new();
    for (kw, args) in tokens {
        let values = args
            .into_iter()
            .filter_map(|arg| match arg {
                lexer::Token::REG(register) =>  Some(ArgValue::REGISTER(register)),
                lexer::Token::LABEL(s)      =>  Some(ArgValue::LABEL(s)),
                lexer::Token::NUMBER(n)     =>  Some(ArgValue::NUMBER(n)),
                lexer::Token::NAME(name)    =>  Some(ArgValue::USE(name)),
                // lexer::Token::(n) =>  Some(Value::OFFSET(n)),
                _ => None
            })
            .collect();
        v.push((kw, values));
    }
    v
}



// 2.3 PSEUDO INSTRUCTION TRANSLATION/EXPANSION

fn expand_pseudos(
    stats: Vec<(lexer::Token, Vec<ArgValue>)>
) -> Vec<(lexer::Token, Vec<ArgValue>)>
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



// 2.4 KEY SPECIALIZATION

fn specialize_keys(
    stats: Vec<(lexer::Token, Vec<ArgValue>)>
) -> Vec<(KeyValue, Vec<ArgValue>)>
{
    let mut v = Vec::new();
    for (kw, args) in stats {
        let key = match kw {
            lexer::Token::OP(o) => {
                Some(KeyValue::OP(o))
            },
            lexer::Token::DIRECTIVE(d) => {
                Some(KeyValue::DIRECTIVE(d))
            },
            lexer::Token::LABEL(l) => {
                Some(KeyValue::LABEL(l))
            },
            lexer::Token::SECTION(s) => {
                match s.as_str() {
                    "text" => {
                        Some(KeyValue::SECTION(AssemblySection::TEXT))
                    },
                    "data" => {
                        Some(KeyValue::SECTION(AssemblySection::DATA))
                    },
                    "bss" => {
                        Some(KeyValue::SECTION(AssemblySection::BSS))
                    },
                    _ => {
                        Some(KeyValue::SECTION(AssemblySection::CUSTOM(s)))
                    }
                }
            },
            _ => {
                None
            }
        };
        if let Some(k) = key {
            v.push((k, args));
        }
    }
    v
}



// TODO: 2.5  DIRECTIVE EXPANSION



// TODO: 2.6 SECTION HANDLING



// 2.7 GENERATING ADDRESSES

fn gen_addresses(
    stats: Vec<(KeyValue, Vec<ArgValue>)>
) -> Vec<(usize, KeyValue, Vec<ArgValue>)>
{
    let mut v = Vec::new();
    for (idx, (kw, args)) in stats.into_iter().enumerate() {
        v.push((idx * 4, kw, args))
    }
    v
}



// 2.8  SYMBOLIC LABEL TO ADDRESS

use std::collections::HashMap;

fn resolve_labels(
    stats: &Vec<(usize, KeyValue, Vec<ArgValue>)>
) -> HashMap<String, usize>
{
    let mut v = HashMap::new();
    for stat in stats {
        match &stat.1 {
            KeyValue::LABEL(s) => {
                v.insert(s.clone(), stat.0);
            },
            _ => {
            }
        }
    }
    v
}

fn replace_symbolic_labels(
    stats: Vec<(usize, KeyValue, Vec<ArgValue>)>,
    symbmap: HashMap<String, usize>
) -> Vec<(usize, KeyValue, Vec<ArgValue>)>
{
    let mut v = Vec::new();
    let mut new_args = Vec::new();
    for (addr, kw, args) in stats {
        for arg in args {
            match arg {
                ArgValue::LABEL(s) => {
                    if let Some(v) = symbmap.get(&s) {
                        new_args.push(ArgValue::OFFSET(*v));
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



// 3 CONVERTING ARGUMENTS TO ACTUAL NUMBERS

fn args_to_numbers(
    stats: Vec<(usize, KeyValue, Vec<ArgValue>)>
) ->  Vec<AssemblyInstruction>
{
    let mut v = Vec::new();
    for (addr, kw, args) in stats {
        let new_args = args.iter().filter_map(|arg| match *arg {
            ArgValue::NUMBER(n) => Some(n),
            ArgValue::REGISTER(register) => Some(register.id().into()),
            ArgValue::OFFSET(o) => Some(o.try_into().unwrap()),
            ArgValue::USE(_)    => None,
            ArgValue::LABEL(_)  => panic!(),
        }).collect();
        v.push(AssemblyInstruction {
            addr,
            key: kw,
            args: new_args
        });
    }
    v
}

pub fn parse(
    tokens: Vec<lexer::Token>
) ->  Vec<AssemblyInstruction>
{
    let stats = group_tokens(tokens);
    let stats = map_args_to_values(stats);
    let stats = expand_pseudos(stats);
    let stats = specialize_keys(stats);
    let stats = gen_addresses(stats);
    let m =  resolve_labels(&stats);
    let stats = replace_symbolic_labels(stats, m);
    args_to_numbers(stats)
}
