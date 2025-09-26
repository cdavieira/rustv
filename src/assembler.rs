use crate::lang::highassembly::{
    ArgValue,
    SectionName,
    KeyValue,
    GenericBlock,
    GenericLine,
};
use crate::lang::lowassembly::{
    EncodableKey,
    EncodableLine,
    PositionedEncodableBlock,
    PositionedEncodedBlock,
};
use std::collections::HashMap;

pub trait Assembler {
    type Input;
    fn assemble(&self, instructions: Self::Input) -> AssemblerTools ;
}





/**/

#[derive(Debug)]
pub struct Symbol {
    pub(crate) section: SectionName,
    pub(crate) relative_address: usize,
    pub(crate) length: usize,
    pub(crate) scope: String,
}

#[derive(Debug)]
pub struct Section {
    pub(crate) address: usize,
    pub(crate) name: SectionName,
}

#[derive(Debug)]
pub struct RelocationEntry {
    pub(crate) address: usize,
    pub(crate) addend: i32,
}

#[derive(Debug)]
pub struct AssemblerTools {
    pub(crate) metadata: Option<GenericBlock>,
    pub(crate) sections: HashMap<String, Section>,
    pub(crate) symbols:  HashMap<String, Symbol>,
    pub(crate) strings:  Vec<String>,
    pub(crate) relocations:  HashMap<String, RelocationEntry>,
    pub(crate) blocks: Vec<PositionedEncodedBlock>,
}

impl AssemblerTools {
    fn section_words(&self, name: SectionName) -> Vec<u32> {
        let text_sections_data: Vec<Vec<u32>> = self.blocks
            .iter()
            .filter_map(|block| {
                if block.name == name {
                    let data: Vec<_> = block.instructions
                        .iter()
                        .map(|i| i.data.clone())
                        .flatten()
                        .collect();
                    Some(data)
                } else {
                    None
                }
            })
            .collect();
        text_sections_data
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn text_section_words(&self) -> Vec<u32> {
        self.section_words(SectionName::Text)
    }

    pub fn data_section_words(&self) -> Vec<u32> {
        self.section_words(SectionName::Data)
    }
}





fn extract_metadata(
    blocks: &mut Vec<GenericBlock>
) -> Option<GenericBlock>
{
    let pair = blocks
        .iter()
        .enumerate()
        .find(|(_, block)| block.name == SectionName::Metadata);
    if let Some((index, _)) = pair {
        Some(blocks.remove(index))
    }
    else {
        None
    }
}

fn gen_positions(
    blocks: Vec<GenericBlock>
) -> Vec<PositionedGenericBlock>
{
    blocks.
        into_iter()
        .map(|block| {
            PositionedGenericBlock {
                address: 0,
                name: block.name,
                lines: block.lines
                    .into_iter()
                    .map(|line| {
                        PositionedGenericLine {
                            relative_address: 0,
                            line,
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

// 2.1 Generating the start address of each section

#[derive(Debug)]
pub struct PositionedGenericLine {
    relative_address: usize, /* relative address */
    line: GenericLine,
}

#[derive(Debug)]
pub struct PositionedGenericBlock {
    address: usize,
    name: SectionName,
    lines: Vec<PositionedGenericLine>,
}

fn gen_section_address(
    blocks: Vec<PositionedGenericBlock>,
    initial_addr: usize,
    block_offset: usize,
) -> Vec<PositionedGenericBlock>
{
    //OBS: if block_offset % 4 != 0, then this could lead to section alignment problems
    let mut new_blocks = Vec::new();
    let mut next_block_address = initial_addr;
    for block in blocks {
        let block_size: usize = block.lines
            .iter()
            .map(|line| line.line.size_bytes_with_alignment(4usize))
            .sum();
        new_blocks.push(PositionedGenericBlock{
            address: next_block_address,
            ..block
        });
        next_block_address += block_size + block_offset;
    }
    new_blocks
}

// 2.2 Generating the relative address of each instruction

fn gen_line_address(blocks: Vec<PositionedGenericBlock>) -> Vec<PositionedGenericBlock> {
    let mut new_blocks = Vec::new();
    for block in blocks {
        let mut relative_address = 0;
        let lines_with_address = block.lines
            .into_iter()
            .map(|line| {
                let new_line = PositionedGenericLine {
                    relative_address,
                    ..line
                };
                relative_address += new_line.line.size_bytes_with_alignment(4);
                new_line
            })
            .collect();
        new_blocks.push(PositionedGenericBlock{
            lines: lines_with_address,
            ..block
        });
    }
    new_blocks
}

// 2.3 Generating section table

fn gen_section_table(sections: &Vec<PositionedGenericBlock>) -> HashMap<String, Section> {
    let mut map = HashMap::new();
    for section in sections {
        let value = Section {
            name: section.name.clone(),
            address: section.address,
        };
        map.insert(section.name.default_name(), value);
    }
    map
}

// 2.4 Generating symbol table

fn gen_symbol_table(sections: &Vec<PositionedGenericBlock>) -> HashMap<String, Symbol> {
    let mut v = HashMap::new();
    for section in sections {
        for line in &section.lines {
            match &line.line.keyword {
                KeyValue::Label(s) => {
                    let symbol_size = section
                        .lines
                        .iter()
                        .find_map(|line| {
                            match line.line.keyword {
                                KeyValue::AssemblyDirective(_) => Some(line.line.size_bytes_with_alignment(1)),
                                _ => None,
                            }
                        });
                    let value = Symbol {
                        section: section.name.clone(),
                        relative_address: line.relative_address,
                        scope: String::from("File"),
                        length: match symbol_size {
                                Some(size) => size,
                                None => 0,
                            },
                    };
                    v.insert(s.clone(), value);
                },
                _ => {
                }
            }
        }
    }
    v
}

// 2.5 Generating string table

fn gen_string_table(_sections: &Vec<PositionedGenericBlock>) -> Vec<String> {
    let v = Vec::new();
    // for section in sections {
    //     for line in &section.lines {
    //         if let KeyValue::AssemblyDirective(_) = line.line.keyword {
    //         }
    //     }
    // }
    v
}

// 2.X Generating relocatable table

fn gen_relocation_table(
    blocks: &Vec<PositionedGenericBlock>,
    symbols: &HashMap<String, Symbol>,
    sections: &HashMap<String, Section>,
) -> HashMap<String, RelocationEntry> {
    let mut relocation_table = HashMap::new();
    for section in blocks {
        if section.name != SectionName::Text {
            continue;
        }
        for line in &section.lines {
            for arg in &line.line.args {
                match arg {
                    ArgValue::UseHi(s, addend) => {
                        let res = get_symb_addrs(&s, symbols, sections);
                        if let Ok((_, symb_sect)) = res {
                            if symb_sect.name != section.name {
                                let relocation = RelocationEntry {
                                    address: line.relative_address,
                                    addend: *addend,
                                };
                                relocation_table.insert(s.clone(), relocation);
                            }
                        };
                    },
                    _ => {
                    },
                }
            }
        }
    }
    relocation_table 
}

// 2.6 Resolving symbols

fn get_symb_addrs<'a, 'b>(
    symb: &str,
    symbols: &'a HashMap<String, Symbol>,
    sections: &'b HashMap<String, Section>,
) -> Result<(&'a Symbol, &'b Section), ()> {
    let Some(symb) = symbols.get(symb) else {
        return Err(());
    };
    let Some(section) = sections.get(&symb.section.default_name()) else {
        return Err(());
    };
    Ok((symb, section))
}

fn compute_offset(
    src_section: usize,
    src_line: usize,
    symbol: &Symbol,
    symbol_section: &Section,
) -> i32
{
    let line_faddr: i32 = (src_section + src_line)
        .try_into()
        .unwrap();
    let symb_faddr: i32 = (symbol_section.address + symbol.relative_address)
        .try_into()
        .unwrap();
    symb_faddr - line_faddr
}

fn resolve_symbols(
    blocks: Vec<PositionedGenericBlock>,
    symbols: &HashMap<String, Symbol>,
    sections: &HashMap<String, Section>
) -> Vec<PositionedGenericBlock>
{
    let mut resolved_sections = Vec::new();
    for section in blocks {
        let mut new_lines = Vec::new();
        for line in section.lines {
            let mut new_args = Vec::new();
            for arg in line.line.args {
                match arg {
                    ArgValue::Use(s, addend) => {
                        let res = get_symb_addrs(&s, symbols, sections);
                        if let Ok((symb, symb_sect)) = res {
                            let offset = compute_offset(
                                section.address, line.relative_address, symb, symb_sect);
                            new_args.push(ArgValue::Number(offset + addend));
                        }
                    },
                    ArgValue::UseHi(s, addend) => {
                        let res = get_symb_addrs(&s, symbols, sections);
                        if let Ok((symb, symb_sect)) = res {
                            let offset = compute_offset(
                                section.address, line.relative_address, symb, symb_sect);
                            let hi = (offset >> 12) & 0b11111_11111_11111_11111;
                            new_args.push(ArgValue::Number(hi + addend));
                        }
                    },
                    ArgValue::UseLo(s, addend) => {
                        let res = get_symb_addrs(&s, symbols, sections);
                        if let Ok((symb, symb_sect)) = res {
                            let offset = compute_offset(
                                section.address, line.relative_address, symb, symb_sect);
                            let lo = offset & 0b1111_1111_1111;
                            new_args.push(ArgValue::Number(lo + addend));
                        }
                    },
                    _ => {
                        new_args.push(arg.clone());
                    }
                }
            }
            new_lines.push(PositionedGenericLine{
                line: GenericLine {
                    keyword: line.line.keyword,
                    args: new_args
                },
                ..line
            });
        }
        resolved_sections.push(PositionedGenericBlock{
            lines: new_lines,
            ..section
        });
    }
    resolved_sections
}

// 2.7 Converting all arguments to numbers

fn args_to_numbers(blocks: Vec<PositionedGenericBlock>) -> Vec<PositionedEncodableBlock> {
    let mut sections = Vec::new();
    let mut instructions = Vec::new();
    for block in blocks {
        for line in block.lines {
            match line.line.keyword {
                KeyValue::Op(op) => {
                    let args: Vec<i32> = line
                        .line
                        .args
                        .iter()
                        .filter_map(|arg| arg.to_number())
                        .collect();
                    instructions.push(EncodableLine {
                        key: EncodableKey::Op(op),
                        args
                    });
                },
                KeyValue::AssemblyDirective(d) => {
                    let args: Vec<i32> = line.line.args
                        .into_iter()
                        .filter_map(|arg| arg.to_number())
                        .collect();
                    instructions.push(EncodableLine {
                        key: EncodableKey::Directive(d),
                        args,
                    });
                },
                _ => {
                },
            };
        }
        sections.push(PositionedEncodableBlock {
            addr: block.address,
            name: block.name,
            instructions: instructions.drain(..).collect(),
        });
    }
    sections
}

fn encode_blocks(blocks: Vec<PositionedEncodableBlock>) -> Vec<PositionedEncodedBlock> {
    let mut new_blocks = Vec::new();
    for block in blocks {
        // println!("Processing {:?}", &i.key);
        new_blocks.push(PositionedEncodedBlock {
            addr: block.addr,
            name: block.name,
            instructions: block.instructions
                .into_iter()
                .map(|line| line.encode())
                .collect()
        });
    }
    new_blocks
}

pub fn to_u32(mut blocks: Vec<GenericBlock>) -> AssemblerTools {
    let metadata = extract_metadata(&mut blocks);

    let blocks = gen_positions(blocks);
    // println!("{:?}", blocks);

    let blocks = gen_section_address(blocks, 0, 4);
    // println!("{:?}", blocks);

    let blocks = gen_line_address(blocks);
    // println!("{:?}", blocks);
    // dbg!(&blocks);

    let sections = gen_section_table(&blocks);
    let symbols  = gen_symbol_table(&blocks);
    let strings  = gen_string_table(&blocks);
    let relocations = gen_relocation_table(&blocks, &symbols, &sections);
    // dbg!(&sections);
    // dbg!(&symbols);
    // dbg!(&strings);

    let blocks = resolve_symbols(blocks, &symbols, &sections);
    // println!("{:?}", blocks);
    // dbg!(&blocks);

    let blocks = args_to_numbers(blocks);
    // println!("{:?}", blocks);
    // dbg!(&blocks);

    let blocks = encode_blocks(blocks);
    // println!("{:?}", blocks);
    // dbg!(&blocks);

    AssemblerTools {
        metadata,
        sections,
        symbols,
        strings,
        relocations,
        blocks,
    }
}
