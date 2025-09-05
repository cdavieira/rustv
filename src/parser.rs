pub trait Parser {
    type Token;
    type Output;

    fn parse(&self, token: Vec<Self::Token>) -> Self::Output ;
}



/* The following code was written to ease the process of implementing the 'Parser' trait. */


use crate::spec::{
    ArgValue,
    AssemblySectionName,
    KeyValue,
    SemanticLine,
    SemanticBlock
};

use crate::lexer::{
    ToGenericToken,
    GenericToken,
};



//2.0 Generalizing tokens

fn generalize_tokens<T: ToGenericToken>(tokens: Vec<T>) -> Vec<GenericToken> {
    tokens
        .into_iter()
        .filter_map(|token| token.to_generic_token())
        .collect()
}

//2.1 Grouping tokens in lines

// TODO: this can be later simplified by introducing a EOL token to the GenericToken enum,
// which would allow identifying end of lines and therefore lines
fn group_tokens(tokens: Vec<GenericToken>) -> Vec<SemanticLine> {
    let mut token_groups = Vec::new();
    let mut args = Vec::new();
    for token in tokens.into_iter().rev() {
        match token {
            GenericToken::KeyToken(k) =>  {
                let group = SemanticLine {
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
    token_groups.into_iter().rev().collect()
}


// 2.2 Expanding pseudo instructions into groups of real instructions

fn expand_pseudos(lines: Vec<SemanticLine>) -> Vec<SemanticLine> {
    let mut expanded_lines = Vec::new();
    for line in lines {
        match &line.keyword {
            KeyValue::Pseudo(pseudo) => {
                let extra_lines: Vec<SemanticLine> = pseudo
                    .translate(line.args)
                    .into_iter()
                    .map(|(i, a)| {
                            SemanticLine{
                                keyword: KeyValue::Op(i),
                                args: a
                            }
                        }
                    )
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



// 2.3 Expanding directives into bytes

fn expand_assembly_directives(lines: Vec<SemanticLine>) -> Vec<SemanticLine> {
    let mut new_lines = Vec::new();
    for line in lines {
        match &line.keyword {
            KeyValue::AssemblyDirective(d) => {
                let new_args: Vec<ArgValue> = d.translate(&line.args)
                    .into_iter()
                    .map(|a| ArgValue::Byte(a))
                    .collect();
                new_lines.push(SemanticLine{
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



// 2.4 Grouping instructions into sections

fn group_blocks(lines: Vec<SemanticLine>) -> Vec<SemanticBlock> {
    let mut blocks = vec![];
    let mut block_lines = vec![];
    let mut metadata = SemanticBlock{
        name: AssemblySectionName::Metadata,
        lines: Vec::new(),
    };
    for line in lines.into_iter().rev() {
        match line.keyword {
            KeyValue::Section(s) => {
                blocks.push(SemanticBlock{
                    name: s,
                    lines: block_lines.drain(..).rev().collect(),
                });
            },
            KeyValue::LinkerDirective(_) => metadata.lines.push(line),
            _ => {
                block_lines.push(line);
            }
        }
    }
    //If there are instructions left without a session, wrap them in a text section 
    if block_lines.len() > 0 {
        blocks.push(SemanticBlock{
            name: AssemblySectionName::Text,
            lines: block_lines.drain(..).rev().collect(),
        });
    }
    blocks.push(metadata);
    blocks.into_iter().rev().collect()
}

fn merge_blocks(blocks: Vec<SemanticBlock>) -> Vec<SemanticBlock> {
    let mut metadata = SemanticBlock{name: AssemblySectionName::Metadata, lines: Vec::new()};
    let mut text = SemanticBlock{name: AssemblySectionName::Text, lines: vec![]};
    let mut data = SemanticBlock{name: AssemblySectionName::Data, lines: vec![]};
    let mut bss  = SemanticBlock{name: AssemblySectionName::Bss, lines: vec![]};
    let mut v = vec![];
    for block in blocks {
        match block.name {
            AssemblySectionName::Text => {
                text.lines.extend(block.lines);
            },
            AssemblySectionName::Data => {
                data.lines.extend(block.lines);
            },
            AssemblySectionName::Bss  => {
                bss.lines.extend(block.lines);
            },
            AssemblySectionName::Metadata  => {
                metadata.lines.extend(block.lines);
            },
            AssemblySectionName::Custom(_) => panic!("Custom sections are not yet implemented :/"),
        }
    }
    v.push(metadata);
    v.push(data);
    v.push(text);
    v.push(bss);
    v
}

pub fn parse<T: ToGenericToken>(tokens: Vec<T>) -> Vec<SemanticBlock> {
    let tokens = generalize_tokens(tokens);
    let groups = group_tokens(tokens);
    let lines  = expand_pseudos(groups);
    let lines  = expand_assembly_directives(lines);
    let blocks = group_blocks(lines);
    let blocks = merge_blocks(blocks);
    blocks
}
