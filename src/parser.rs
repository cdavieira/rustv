pub trait Parser {
    type Token;
    type Output;

    fn parse(&self, token: Vec<Self::Token>) -> Self::Output ;
}

/* The following code was written to ease the process of implementing the 'Parser' trait. */

use crate::lang::highassembly::{
    ArgValue,
    SectionName,
    KeyValue,
    GenericLine,
    GenericBlock
};

use crate::lexer::{
    ToGenericToken,
    GenericToken,
};

//2.1 Converting tokens to their generic representative

fn generalize_tokens<T: ToGenericToken>(tokens: Vec<T>) -> Vec<GenericToken> {
    tokens
        .into_iter()
        .filter_map(|token| token.to_generic_token())
        .collect()
}

//2.2 Grouping tokens in lines

// TODO: introducing an EOL token to the GenericToken enum would allow this aswell
fn group_tokens(tokens: Vec<GenericToken>) -> Vec<GenericLine> {
    let mut token_groups = Vec::new();
    let mut args = Vec::new();
    for token in tokens.into_iter().rev() {
        match token {
            GenericToken::KeyToken(k) =>  {
                let group = GenericLine {
                    id: 0,
                    keyword: k,
                    args: args.drain(..).rev().collect()
                };
                token_groups.push(group);
            },
            GenericToken::ArgToken(a) => {
                args.push(a);
            },
        }
    }
    let lines: Vec<_> = token_groups.into_iter().rev().collect();
    lines
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            GenericLine {
                id: idx,
                ..line
            }
        })
        .collect()
}

// 2.3 Expanding pseudo instructions into groups of real instructions

fn expand_pseudos(lines: Vec<GenericLine>) -> Vec<GenericLine> {
    let mut expanded_lines = Vec::new();
    for line in lines {
        match &line.keyword {
            KeyValue::Pseudo(pseudo) => {
                let extra_lines: Vec<GenericLine> = pseudo
                    .translate(line.args)
                    .into_iter()
                    .map(|opcode_line| {
                        GenericLine {
                            id: line.id,
                            keyword: KeyValue::Op(opcode_line.keyword),
                            args: opcode_line.args,
                        }
                    })
                    .collect()
                ;
                expanded_lines.extend(extra_lines);
            },
            _ => {
                expanded_lines.push(line);
            }
        }
    }
    expanded_lines
}

// 2.4 Expanding directives into bytes

fn expand_assembly_directives(lines: Vec<GenericLine>) -> Vec<GenericLine> {
    let mut new_lines = Vec::new();
    for line in lines {
        match &line.keyword {
            KeyValue::AssemblyDirective(d) => {
                let new_args: Vec<ArgValue> = d.translate(&line.args)
                    .into_iter()
                    .map(|a| ArgValue::Byte(a))
                    .collect();
                new_lines.push(GenericLine{
                    id: line.id,
                    keyword: line.keyword,
                    args: new_args
                });
            },
            _ => {
                new_lines.push(line);
            }
        }
    }
    new_lines
}

// 2.5 Grouping instructions into sections

fn group_lines(lines: Vec<GenericLine>) -> Vec<GenericBlock> {
    let mut blocks = vec![];
    let mut block_lines = vec![];
    let mut metadata = GenericBlock{
        name: SectionName::Metadata,
        lines: Vec::new(),
    };
    for line in lines.into_iter().rev() {
        match line.keyword {
            KeyValue::Section(s) => {
                blocks.push(GenericBlock{
                    name: s,
                    lines: block_lines.drain(..).rev().collect(),
                });
            },
            KeyValue::LinkerDirective(_) => metadata.lines.push(line),
            _ => block_lines.push(line),
        }
    }
    //If there are instructions left without a session, wrap them in a text section 
    if block_lines.len() > 0 {
        blocks.push(GenericBlock{
            name: SectionName::Text,
            lines: block_lines.drain(..).rev().collect(),
        });
    }
    blocks.push(metadata);
    blocks.into_iter().rev().collect()
}

// 2.6 Merging same groups

fn merge_blocks(blocks: Vec<GenericBlock>) -> Vec<GenericBlock> {
    let mut metadata = GenericBlock{name: SectionName::Metadata, lines: Vec::new()};
    let mut text = GenericBlock{name: SectionName::Text, lines: vec![]};
    let mut data = GenericBlock{name: SectionName::Data, lines: vec![]};
    let mut bss  = GenericBlock{name: SectionName::Bss, lines: vec![]};
    let mut v = vec![];
    for block in blocks {
        match block.name {
            SectionName::Text => text.lines.extend(block.lines),
            SectionName::Data => data.lines.extend(block.lines),
            SectionName::Bss  => bss.lines.extend(block.lines),
            SectionName::Metadata  => metadata.lines.extend(block.lines),
            SectionName::Custom(_) => panic!("Custom sections are not yet implemented :/"),
        }
    }
    v.push(metadata);
    v.push(text);
    v.push(data);
    v.push(bss);
    v
}

pub fn tokens_to_lines<T: ToGenericToken>(tokens: Vec<T>) -> Vec<GenericLine> {
    let tokens = generalize_tokens(tokens);
    let groups = group_tokens(tokens);
    let lines  = expand_pseudos(groups);
    let lines  = expand_assembly_directives(lines);
    lines
}

fn lines_to_blocks(lines: Vec<GenericLine>) -> Vec<GenericBlock> {
    let blocks = group_lines(lines);
    let blocks = merge_blocks(blocks);
    blocks
}

pub fn parse<T: ToGenericToken>(tokens: Vec<T>) -> Vec<GenericBlock> {
    let lines  = tokens_to_lines(tokens);
    let blocks = lines_to_blocks(lines);
    blocks
}
