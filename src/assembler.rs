use crate::spec::{AssemblyData, SemanticBlock};

pub trait Assembler {
    type Input;
    fn to_words(&self, instructions: Self::Input) -> AssemblyData ;
}

/**/

use crate::spec::{
    instruction_to_binary, AssemblySectionName, KeyValue, ArgValue, AssemblySection
};
use crate::utils;
use std::collections::HashMap;

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

// 2.5.2 Merging same sections

fn merge_sections(blocks: Vec<SemanticBlock>) -> Vec<SemanticBlock> {
    let mut text = SemanticBlock{address: 0, name: AssemblySectionName::TEXT, lines: vec![]};
    let mut data = SemanticBlock{address: 0, name: AssemblySectionName::DATA, lines: vec![]};
    let mut bss  = SemanticBlock{address: 0, name: AssemblySectionName::BSS, lines: vec![]};
    let mut v = vec![];
    for block in blocks {
        match block.name {
            AssemblySectionName::TEXT => text.lines.extend(block.lines),
            AssemblySectionName::DATA => data.lines.extend(block.lines),
            AssemblySectionName::BSS  => bss.lines.extend(block.lines),
            AssemblySectionName::CUSTOM(_) => panic!("Custom sections are not yet implemented :/"),
        }
    }
    v.push(data);
    v.push(text);
    v.push(bss);
    v
}



// 2.6.1 Generating the start address of each section

fn gen_section_address(blocks: Vec<SemanticBlock>) -> Vec<SemanticBlock> {
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
        new_blocks.push(SemanticBlock{
            address: block_addr,
            ..block
        });
        block_addr += block_len + block_off;
    }
    new_blocks
}



// 2.6.2 Generating the relative address of each instruction

fn gen_line_address(blocks: Vec<SemanticBlock>) -> Vec<SemanticBlock> {
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
                // TODO: What about KeyValue::Directive?
                if let KeyValue::OP(_) = &new_line.keyword {
                    rel_addr += 4;
                }
                new_line
            }).collect();
        v.push(SemanticBlock{
            lines: lines_with_address,
            ..block
        });
    }
    v
}



// 2.7.1 Generating section and symbolic table

fn gen_section_table(sections: &Vec<SemanticBlock>) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for section in sections {
        let key = match &section.name {
            AssemblySectionName::TEXT => ".text".to_string(),
            AssemblySectionName::DATA => ".data".to_string(),
            AssemblySectionName::BSS  => ".bss".to_string(),
            AssemblySectionName::CUSTOM(s) => s.clone(),
        };
        map.insert(key, section.address);
    }
    map
}

fn gen_symbol_table(sections: &Vec<SemanticBlock>) -> HashMap<String, (AssemblySectionName, usize)> {
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
    sections: Vec<SemanticBlock>,
    symbtable: &HashMap<String, (AssemblySectionName, usize)>,
    secttable: &HashMap<String, usize>
) -> Vec<SemanticBlock>
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
        resolved_sections.push(SemanticBlock{
            lines: new_lines,
            ..section
        });
    }
    resolved_sections
}



// 3 Converting all arguments to numbers (i32)

fn args_to_numbers(blocks: Vec<SemanticBlock>) -> Vec<AssemblySection> {
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

pub fn tunnel(blocks: Vec<SemanticBlock>) -> Vec{
    let merged_blocks = merge_sections(blocks);
    let blocks = gen_section_address(merged_blocks);
    let blocks = gen_line_address(blocks);
    // println!("{:?}", blocks);

    let section_table = gen_section_table(&blocks);
    let symbol_table  = gen_symbol_table(&blocks);


    // The only difference between a SectionBlock and an AssemblySection is that the args of an
    // assemblysection are all numbers, while sectionblocks are still unresolved (symbols + labels
    // + other definitions)
    // 1. Move the functions above to the assembler
    // 2. What happens with the type associated then with the resulting output (Vec<SectionBlock>)?
    //    it will need to be shared with the assembler, or it could be standardized
    // 3. Enhance what a symbol is and what a section is and share those types so that the
    //    assembler can work with them
    //    3.1 In 'gen_symbol_table' -> a symbol has at least an address and a section
    // 4. maybe separate the functions which produce metadata/remove directives

    let blocks        = resolve_symbols(blocks, &symbol_table, &section_table);
    // dbg!(&blocks);

    let sections = args_to_numbers(blocks);
    let metadata = metadata
        .into_iter()
        .map(|line| (line.keyword, line.args))
        .collect();
    dbg!(&sections);

    ParserOutput {
        metadata,
        section_table,
        symbol_table,
        sections
    }
}

pub fn to_u32(section: AssemblySection) -> AssemblyData {
    let mut data = Vec::new();

    for i in &section.instructions {
        // println!("Processing {:?}", &i.key);
        match &i.key {
            KeyValue::OP(extension) => {
                let word = instruction_to_binary(extension, &i.args);
                // println!("Turned into {}", word);
                data.push(word);
            },
            KeyValue::DIRECTIVE(_) => {
                let words: Vec<u32> = i.args
                        .iter()
                        .map(|x| *x as u32)
                        .collect();
                // println!("Turned into {:?}", words);
                data.extend(words);
            },
            _ => {}
        }
    }
    // println!("Len 1: {}", data.len());

    let (addr, name) = (section.addr, section.name);
    AssemblyData {
        addr,
        name,
        data,
    }
}
