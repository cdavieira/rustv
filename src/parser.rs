pub trait Parser {
    type Token;
    type Output;

    fn parse(&self, token: Vec<Self::Token>) -> Self::Output ;
}



/* The following code was written to ease the process of implementing the 'Parser' trait. */

use std::collections::HashMap;

use crate::lexer;

use crate::spec::{
    ArgValue,
    AssemblyInstruction,
    AssemblySection,
    AssemblySectionName,
    KeyValue,
};

#[derive(Debug)]
struct GroupedLine {
    keyword: lexer::Token,
    args: Vec<lexer::Token>
}

#[derive(Debug)]
struct SemanticLine {
    rel_addr: usize,
    keyword: KeyValue,
    args: Vec<ArgValue>
}

#[derive(Debug)]
struct SectionBlock {
    address: usize,
    name: AssemblySectionName,
    lines: Vec<SemanticLine>
}

#[derive(Debug)]
pub struct ParserOutput {
    metadata: Vec<(KeyValue, Vec<ArgValue>)>,
    section_table: HashMap<String, usize>,
    symbol_table:  HashMap<String, (AssemblySectionName, usize)>,
    sections: Vec<AssemblySection>,
}

impl<'a> ParserOutput {
    pub fn get_sections(&mut self) -> Vec<AssemblySection> {
        if self.sections.len() < 2 {
            return vec![];
        }
        let text = self.sections.remove(0);
        let data = self.sections.remove(0);
        let bss = self.sections.remove(0);
        vec![text, data, bss]
    }

    pub fn get_all(self)
    -> (
        Vec<(KeyValue, Vec<ArgValue>)>,
        HashMap<String, (AssemblySectionName, usize)>,
        HashMap<String, usize>,
        Vec<AssemblySection>
    )
    {
        (self.metadata, self.symbol_table, self.section_table, self.sections)
    }

    pub fn text_section(&'a self) -> &'a AssemblySection {
        &self.sections[0]
    }

    pub fn data_section(&'a self) -> &'a AssemblySection {
        &self.sections[1]
    }

    pub fn bss_section(&'a self) -> &'a AssemblySection {
        &self.sections[2]
    }

    // pub fn symbol_table(&'a self) -> &'a HashMap<String, usize> {
    //     &self.symbol_table
    // }

    pub fn metadata(&'a self) -> &'a Vec<(KeyValue, Vec<ArgValue>)> {
        &self.metadata
    }

    // pub fn start_symbol(&self) -> usize {
    //     *self.symbol_table.get(&String::from("_start")).unwrap()
    // }
}



//2.1 Grouping tokens in lines

fn group_tokens(tokens: Vec<lexer::Token>) -> Vec<GroupedLine> {
    let mut token_groups = Vec::new();
    let mut args = Vec::new();
    for token in tokens.into_iter().rev() {
        match &token {
            lexer::Token::OP(_)        |
            lexer::Token::PSEUDO(_)    |
            lexer::Token::DIRECTIVE(_) |
            lexer::Token::SECTION      |
            lexer::Token::LABEL(_) =>  {
                let group = GroupedLine {
                    keyword: token,
                    args: args.drain(..).rev().collect()
                };
                token_groups.push(group);
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



// 2.2 Converting the first element of the group into a key and
// the second element into a sequence of arguments
    
fn parse_semantic_lines(groups: Vec<GroupedLine>) -> Vec<SemanticLine> {
    let mut v = Vec::new();
    for group in groups {
        let args: Vec<ArgValue> = group.args
            .into_iter()
            .filter_map(|arg| match arg {
                lexer::Token::REG(register) =>  Some(ArgValue::REGISTER(register)),
                lexer::Token::LABEL(s)      =>  Some(ArgValue::LABEL(s)),
                lexer::Token::NUMBER(n)     =>  Some(ArgValue::NUMBER(n)),
                lexer::Token::NAME(name)    =>  Some(ArgValue::USE(name)),
                lexer::Token::STR(s)        =>  Some(ArgValue::LITERAL(s)),
                // lexer::Token::(n) =>  Some(Value::OFFSET(n)),
                _ => None
            })
            .collect();
        let key = match group.keyword {
            lexer::Token::OP(o)        => Some(KeyValue::OP(o)),
            lexer::Token::DIRECTIVE(d) => Some(KeyValue::DIRECTIVE(d)),
            lexer::Token::LABEL(l)     => Some(KeyValue::LABEL(l)),
            lexer::Token::PSEUDO(p)    => Some(KeyValue::PSEUDO(p)),
            lexer::Token::SECTION      => {
                if let Some(ArgValue::USE(s)) = (&args).get(0) {
                    match s.as_str() {
                        ".text" => Some(KeyValue::SECTION(AssemblySectionName::TEXT)),
                        ".data" => Some(KeyValue::SECTION(AssemblySectionName::DATA)),
                        ".bss"  => Some(KeyValue::SECTION(AssemblySectionName::BSS)),
                        _       => Some(KeyValue::SECTION(AssemblySectionName::CUSTOM(s.to_string())))
                    }
                }
                else {
                    None
                }
            },
            _ => None
        };
        if let Some(keyword) = key {
            let line = SemanticLine{ rel_addr: 0, keyword, args, };
            v.push(line);
        }
    }
    v
}



// 2.3 Expanding pseudo instructions into groups of real instructions

fn expand_pseudos(lines: Vec<SemanticLine>) -> Vec<SemanticLine> {
    let mut expanded_lines = Vec::new();
    for line in lines {
        match &line.keyword {
            KeyValue::PSEUDO(pseudo) => {
                let extra_lines: Vec<SemanticLine> = pseudo
                    .translate(line.args)
                    .into_iter()
                    .map(|(i, a)| {
                            SemanticLine{
                                rel_addr: 0,
                                keyword: KeyValue::OP(i),
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



// 2.4 Expanding directives into bytes or into metadata lines

fn expand_directives(
    lines: Vec<SemanticLine>
) -> (Vec<SemanticLine>, Vec<SemanticLine>)
{
    let mut metadata_lines = Vec::new();
    let mut new_lines = Vec::new();
    for line in lines {
        match &line.keyword {
            KeyValue::DIRECTIVE(d) => {
                let new_args: Vec<ArgValue> = d.translate(&line.args)
                    .into_iter()
                    .map(|a| ArgValue::BYTE(a))
                    .collect();
                if new_args.len() > 0 {
                    new_lines.push(SemanticLine{
                        rel_addr: 0,
                        keyword: line.keyword,
                        args: new_args
                    });
                }
                else {
                    metadata_lines.push(line);
                }
            },
            _ => {
                new_lines.push(line);
            }
        }
    }
    (metadata_lines, new_lines)
}



// 2.5.1 Grouping instructions into sections

fn group_sections(lines: Vec<SemanticLine>) -> Vec<SectionBlock> {
    let mut blocks = vec![];
    let mut block_lines = vec![];
    for line in lines.into_iter().rev() {
        match line.keyword {
            KeyValue::SECTION(s) => {
                blocks.push(SectionBlock{
                    address: 0,
                    name: s,
                    lines: block_lines.drain(..).rev().collect(),
                });
            },
            _ => {
                block_lines.push(line);
            }
        }
    }
    if block_lines.len() > 0 {
        blocks.push(SectionBlock{
            address: 0,
            name: AssemblySectionName::TEXT,
            lines: block_lines.drain(..).rev().collect(),
        });
    }
    blocks.into_iter().rev().collect()
}



// 2.5.2 Merging same sections

fn merge_sections(blocks: Vec<SectionBlock>) -> Vec<SectionBlock> {
    let mut text = SectionBlock{address: 0, name: AssemblySectionName::TEXT, lines: vec![]};
    let mut data = SectionBlock{address: 0, name: AssemblySectionName::DATA, lines: vec![]};
    let mut bss  = SectionBlock{address: 0, name: AssemblySectionName::BSS, lines: vec![]};
    let mut v = vec![];
    for block in blocks {
        match block.name {
            AssemblySectionName::TEXT => {
                text.lines.extend(block.lines);
            },
            AssemblySectionName::DATA => {
                data.lines.extend(block.lines);
            },
            AssemblySectionName::BSS  => {
                bss.lines.extend(block.lines);
            },
            AssemblySectionName::CUSTOM(_) => panic!("Custom sections are not yet implemented :/"),
        }
    }
    v.push(data);
    v.push(text);
    v.push(bss);
    v
}



// 2.6.1 Generating the start address of each section

fn gen_section_address(blocks: Vec<SectionBlock>) -> Vec<SectionBlock> {
    let mut new_blocks = Vec::new();
    let mut block_addr = 0;
    let mut block_len: usize;
    //OBS: if block_off % 4 != 0, then this could lead to section alignment problems
    let block_off = 0x4;
    for block in blocks {
        block_len = block.lines
            .iter()
            .map(|item| match item.keyword {
                KeyValue::OP(_)        => 4usize,
                KeyValue::DIRECTIVE(_) => {
                    let len = item.args.len();
                    let exceeding = len % 4;
                    //ensure word alignment for sections
                    let pad = if exceeding > 0 { 4 - exceeding } else { 0 };
                    len + pad
                },
                _ => 0usize
            })
            .sum();
        new_blocks.push(SectionBlock{
            address: block_addr,
            ..block
        });
        block_addr += block_len + block_off;
    }
    new_blocks
}



// 2.6.2 Generating the relative address of each instruction

fn gen_line_address(blocks: Vec<SectionBlock>) -> Vec<SectionBlock> {
    let mut v = Vec::new();
    for block in blocks {
        let mut rel_addr = 0;
        let lines_with_address = block.lines
            .into_iter()
            .map(|line| {
                let new_line = SemanticLine{
                    rel_addr,
                    ..line
                };
                if let KeyValue::OP(_) = &new_line.keyword {
                    rel_addr += 4;
                }
                new_line
            }).collect();
        v.push(SectionBlock{
            lines: lines_with_address,
            ..block
        });
    }
    v
}



// 2.7.1 Generating section and symbolic table

fn gen_section_table(sections: &Vec<SectionBlock>) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for section in sections {
        match &section.name {
            AssemblySectionName::TEXT => {
                map.insert(".text".to_string(), section.address);
            },
            AssemblySectionName::DATA => {
                map.insert(".data".to_string(), section.address);
            }
            AssemblySectionName::BSS  => {
                map.insert(".bss".to_string(), section.address);
            }
            AssemblySectionName::CUSTOM(s) => {
                map.insert(s.clone(), section.address);
            }
        }
    }
    map
}

fn gen_symbol_table(sections: &Vec<SectionBlock>) -> HashMap<String, (AssemblySectionName, usize)> {
    let mut v = HashMap::new();
    for section in sections {
        for line in &section.lines {
            match &line.keyword {
                KeyValue::LABEL(s) => {
                    v.insert(s.clone(), (section.name.clone(), line.rel_addr));
                },
                _ => {
                }
            }
        }
    }
    v
}

// 2.7.2 Resolving symbols

fn get_symb_addrs(
    symb: &str,
    symbtable: &HashMap<String, (AssemblySectionName, usize)>,
    secttable: &HashMap<String, usize>,
) -> Result<(usize, usize), String> {
    if let Some((symb_sec, symb_val)) = symbtable.get(symb) {
        let symb_sec_key = match symb_sec {
            AssemblySectionName::TEXT => ".text".to_string(),
            AssemblySectionName::DATA => ".data".to_string(),
            AssemblySectionName::BSS  => ".bss".to_string(),
            AssemblySectionName::CUSTOM(s) => s.clone(),
        };
        let symb_sec_addr = secttable.get(&symb_sec_key).unwrap();
        Ok((*symb_sec_addr, *symb_val))
    }
    else {
        let errmsg = std::format!("Couldnt find {} in the symb map", symb);
        Err(errmsg)
    }
}

fn handle_symb(res: Result<(usize, usize), String>, mut f: impl FnMut(usize, usize) -> ()) {
    match res {
        Ok((symb_sec_addr, symb_rel_addr)) => {
            f(symb_sec_addr, symb_rel_addr);
        },
        Err(errmsg) => {
            eprintln!("{}", errmsg);
        }
    }
}

fn resolve_symbols(
    sections: Vec<SectionBlock>,
    symbtable: &HashMap<String, (AssemblySectionName, usize)>,
    secttable: &HashMap<String, usize>
) -> Vec<SectionBlock>
{
    let mut resolved_sections = Vec::new();
    for section in sections {
        let mut new_lines = Vec::new();
        for line in section.lines {
            let mut new_args = Vec::new();
            for arg in line.args {
                match arg {
                    //TODO: this could be a problem if the jump is attempted to a symbol in
                    //another section! We need to somehow store the section associated with the
                    //symbol and add some logic here to deal with this
                    ArgValue::USE(s) => {
                        let res = get_symb_addrs(&s, symbtable, secttable);
                        handle_symb(res, |symb_sec_addr, symb_rel_addr| {
                            let start : i32 = line.rel_addr.try_into().unwrap();
                            let end   : i32 = symb_rel_addr.try_into().unwrap();
                            let offset: i32 = end - start;
                            new_args.push(ArgValue::OFFSET(symb_sec_addr, offset));
                        });
                    },
                    ArgValue::USEHI(s) => {
                        let res = get_symb_addrs(&s, symbtable, secttable);
                        handle_symb(res, |symb_sec_addr, symb_rel_addr| {
                            let start : i32 = line.rel_addr.try_into().unwrap();
                            let end   : i32 = symb_rel_addr.try_into().unwrap();
                            let offset: i32 = end - start;
                            let hi = (offset >> 12) & 0b11111_11111_11111_11111;
                            // println!("{} {} {} {} {}", symb_sec_addr, end, start, offset, hi);
                            new_args.push(ArgValue::OFFSET(symb_sec_addr, hi));
                        });
                    },
                    ArgValue::USELO(s) => {
                        let res = get_symb_addrs(&s, symbtable, secttable);
                        handle_symb(res, |symb_sec_addr, symb_rel_addr| {
                            let start : i32 = line.rel_addr.try_into().unwrap();
                            let end   : i32 = symb_rel_addr.try_into().unwrap();
                            let offset: i32 = end - start;
                            let lo = offset & 0b1111_1111_1111;
                            new_args.push(ArgValue::OFFSET(symb_sec_addr, lo));
                        });
                    },
                    _ => {
                        new_args.push(arg);
                    }
                }
            }
            new_lines.push(SemanticLine{
                args: new_args,
                ..line
            });
        }
        resolved_sections.push(SectionBlock{
            lines: new_lines,
            ..section
        });
    }
    resolved_sections
}



// 3 Converting all arguments to numbers (i32)

fn args_to_numbers(blocks: Vec<SectionBlock>) -> Vec<AssemblySection> {
    let mut sections = Vec::new();
    let mut instructions = Vec::new();
    for block in blocks {
        for line in block.lines {
            let new_args: Vec<i32> = match &line.keyword {
                KeyValue::OP(_) => {
                    line.args.iter().filter_map(|arg| match *arg {
                        ArgValue::REGISTER(register) => Some(register.id().into()),
                        ArgValue::NUMBER(n) => Some(n),
                        ArgValue::OFFSET(abs_addr, rel_addr) => {
                            let abs_addr_unsafe = TryInto::<i32>::try_into(abs_addr)
                                .expect("Fail when converting absolute address to relative");
                            let final_addr: i32 = abs_addr_unsafe + rel_addr;
                            Some(final_addr)
                        },
                        ArgValue::BYTE(b)   => Some(b.try_into().unwrap()),
                        _ => panic!(),
                    }).collect()
                },
                KeyValue::DIRECTIVE(_) => {
                    let values: Vec<i32> = line.args
                        .into_iter()
                        .filter_map(
                            |v| match v {
                                ArgValue::BYTE(b) => Some(b.into()),
                                _ => None,
                            }
                        ).collect();
                    if values.len() > 3 {
                        let (b0, b1, b2, b3) = (values[0], values[1], values[2], values[3]);
                        let n: i32 = (b3 << 24) | (b2 << 16) | (b1 << 8) | b0;
                        vec![n]
                    }
                    else {
                        vec![]
                    }
                },
                _ => {
                    vec![]
                },
            };
            instructions.push(AssemblyInstruction {
                addr: line.rel_addr,
                key: line.keyword,
                args: new_args
            });
        }
        sections.push(AssemblySection {
            addr: block.address,
            name: block.name,
            instructions: instructions.drain(..).collect(),
        });
    }
    sections
}

pub fn parse(tokens: Vec<lexer::Token>) -> ParserOutput {
    let token_groups   = group_tokens(tokens);
    let semantic_lines = parse_semantic_lines(token_groups);
    let no_pseudos     = expand_pseudos(semantic_lines);
    let (metadata, no_directives) = expand_directives(no_pseudos);

    let sparsed_blocks = group_sections(no_directives);
    let grouped_blocks = merge_sections(sparsed_blocks);
    let blocks = gen_section_address(grouped_blocks);
    let blocks = gen_line_address(blocks);
    // println!("{:?}", blocks);

    let section_table = gen_section_table(&blocks);
    let symbol_table  = gen_symbol_table(&blocks);
    let blocks        = resolve_symbols(blocks, &symbol_table, &section_table);
    // dbg!(&blocks);

    let sections = args_to_numbers(blocks);
    let metadata = metadata
        .into_iter()
        .map(|line| (line.keyword, line.args))
        .collect();
    // dbg!(&sections);

    ParserOutput {
        metadata,
        section_table,
        symbol_table,
        sections
    }
}
