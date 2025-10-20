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
    pub(crate) id: usize,
    pub(crate) address: usize,
    pub(crate) addend: i32,
}

#[derive(Debug)]
pub struct AssemblerTools {
    pub(crate) metadata: Option<GenericBlock>,
    pub(crate) sections: HashMap<String, Section>,
    pub(crate) symbols:  HashMap<String, Symbol>,
    pub(crate) strings:  Vec<String>,
    pub(crate) relocations:  HashMap<String, Vec<RelocationEntry>>,
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



/**/

#[derive(Debug)]
pub struct PositionedGenericLine {
    root_relative_address: usize,
    relative_address: usize,
    line: GenericLine,
}

#[derive(Debug)]
pub struct PositionedGenericBlock {
    address: usize,
    name: SectionName,
    lines: Vec<PositionedGenericLine>,
}




// 2.1 Extracting the metadata section
//   The metadata section is intended to:
//    1. store directives that modify the visibility of labels
//    2. the start address (yet to be supported)

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

// 2.2 Casting the 'GenericBlock' type to its sibling type (which can store an address for each section)

fn cast_generic_to_positioned_blocks(
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
                            root_relative_address: 0,
                            relative_address: 0,
                            line,
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

// 2.3 Generating the start address of each section

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
            .map(|line| line.line.size_bytes_at_word_boundary())
            .sum();
        new_blocks.push(PositionedGenericBlock{
            address: next_block_address,
            ..block
        });
        next_block_address += block_size + block_offset;
    }
    new_blocks
}

// 2.4 Generating the relative address of each instruction

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
                // relative_address += new_line.line.size_bytes_unaligned();
                relative_address += new_line.line.size_bytes_at_word_boundary();
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

// 2.5 Generating the so called 'root relative address' of each line
//   The ideia behind this 'root' is that some lines of the program might have been generated
//   because of the expansion of other lines. For example:
//     The instruction:
//       li t1, 2025
//     Gets expanded into:
//       lui  t1, 1
//       addi t1, 1
//   The 'root relative address' stores the address of all lines before their supposed expansion.
//   For the previous example, the 'root relative address' for the 'lui' and 'addi' lines are going
//   to be the same (since both were generated from the 'li' line).
//   This is important later on when creating a relocation

fn gen_root_line_address(blocks: Vec<PositionedGenericBlock>) -> Vec<PositionedGenericBlock> {
    let mut new_blocks = Vec::new();
    for block in blocks {
        let mut line_map: HashMap<usize, usize> = HashMap::new();
        for line in &block.lines {
            if !line_map.contains_key(&line.line.id) {
                line_map.insert(line.line.id, line.relative_address);
            }
        }
        let new_lines = block.lines
            .into_iter()
            .map(|line| PositionedGenericLine {
                    root_relative_address: *line_map.get(&line.line.id).unwrap(),
                    ..line
                }
            )
            .collect();
        new_blocks.push(PositionedGenericBlock {
            lines: new_lines,
            ..block
        })
    }
    new_blocks
}

// 2.6 Generating section table

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

// 2.7 Generating the symbol table

fn gen_symbol_table(sections: &Vec<PositionedGenericBlock>) -> HashMap<String, Symbol> {
    let mut v = HashMap::new();
    for section in sections {
        let mut it = section.lines.iter();
        while let Some(line) = it.next() {
            match &line.line.keyword {
                KeyValue::Label(s) => {
                    if !v.contains_key(s) {
                        let symbol_size = match it.next() {
                            Some(next_line) => {
                                match next_line.line.keyword {
                                    KeyValue::AssemblyDirective(_) => {
                                        line.line.size_bytes_unaligned()
                                    },
                                    _ => 0usize,
                                }
                            },
                            None => 0usize
                        };
                        let value = Symbol {
                            section: section.name.clone(),
                            relative_address: line.relative_address,
                            scope: String::from("File"),
                            length: symbol_size,
                        };
                        v.insert(s.clone(), value);
                    }
                },
                _ => {
                }
            }
        }
    }
    v
}

// 2.8 Generating the string table

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

// 2.9 Generating the relocation table

fn gen_relocation_table(
    blocks: &Vec<PositionedGenericBlock>,
    symbols: &HashMap<String, Symbol>,
    sections: &HashMap<String, Section>,
) -> HashMap<String, Vec<RelocationEntry>> {
    let mut relocation_table = HashMap::new();
    let mut rel_count = 0;
    for section in blocks {
        if section.name != SectionName::Text {
            continue;
        }
        for line in &section.lines {
            for arg in &line.line.args {
                match arg {
                    ArgValue::UseHi(s, addend) => {
                        if let Ok((_, symb_sect)) = get_symb_addrs(&s, symbols, sections) {
                            let refers_external_symbol = symb_sect.name != section.name;
                            if refers_external_symbol {
                                let relname = s.clone();
                                let relocation = RelocationEntry {
                                    id: rel_count,
                                    address: line.relative_address,
                                    addend: *addend,
                                };
                                rel_count += 1;
                                relocation_table
                                    .entry(relname)
                                    .or_insert(Vec::new())
                                    .push(relocation);
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

// 2.10 Resolving symbols

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
    addend: i32,
) -> i32
{
    let line_faddr: i32 = (src_section + src_line)
        .try_into()
        .unwrap();
    let symb_faddr: i32 = (symbol_section.address + symbol.relative_address)
        .saturating_add_signed(addend as isize)
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
                                section.address, line.root_relative_address, symb, symb_sect, addend);
                            new_args.push(ArgValue::Number(offset));
                        }
                    },
                    ArgValue::UseHi(s, addend) => {
                        let res = get_symb_addrs(&s, symbols, sections);
                        if let Ok((symb, symb_sect)) = res {
                            let offset = compute_offset(
                                section.address, line.root_relative_address, symb, symb_sect, addend);
                            let hi = (offset >> 12) & 0b11111_11111_11111_11111;
                            new_args.push(ArgValue::Number(hi));
                        }
                    },
                    ArgValue::UseLo(s, addend) => {
                        let res = get_symb_addrs(&s, symbols, sections);
                        if let Ok((symb, symb_sect)) = res {
                            let offset = compute_offset(
                                section.address, line.root_relative_address, symb, symb_sect, addend);
                            let lo = offset & 0b1111_1111_1111;
                            new_args.push(ArgValue::Number(lo));
                        }
                    },
                    _ => {
                        new_args.push(arg.clone());
                    }
                }
            }
            new_lines.push(PositionedGenericLine{
                line: GenericLine {
                    args: new_args,
                    ..line.line
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

// 2.11 Converting all arguments to numbers

fn generic_to_encodable_lines(lines: Vec<PositionedGenericLine>) -> Vec<EncodableLine> {
    let mut new_lines = Vec::new();
    for line in lines {
        let kw = line.line.keyword;
        let args = line.line.args;
        match kw {
            KeyValue::Op(op) => {
                let args: Vec<i32> = args
                    .iter()
                    .filter_map(|arg| arg.to_number())
                    .collect();
                new_lines.push(EncodableLine {
                    file_pos: line.line.file_pos,
                    key: EncodableKey::Op(op),
                    args
                });
            },
            KeyValue::AssemblyDirective(d) => {
                let handle_endian = match d.datatype().size_bytes() {
                    1 => i32::from_le_bytes,
                    _ => i32::from_le_bytes,
                };
                let args: Vec<u8> = args
                    .iter()
                    .filter_map(|arg| arg.to_number())
                    .map(|n| n as u8)
                    .collect();
                let args: Vec<i32> = args
                    .chunks(4)
                    .map(|chunk| {
                        let b = [
                            chunk.get(0).map_or(0u8, |v| *v),
                            chunk.get(1).map_or(0u8, |v| *v),
                            chunk.get(2).map_or(0u8, |v| *v),
                            chunk.get(3).map_or(0u8, |v| *v)
                        ];
                        handle_endian(b)
                    })
                    .collect();
                new_lines.push(EncodableLine {
                    file_pos: line.line.file_pos,
                    key: EncodableKey::Directive(d),
                    args
                });
            },
            _ => {}
        }
    }
    new_lines
}

fn args_to_numbers(blocks: Vec<PositionedGenericBlock>) -> Vec<PositionedEncodableBlock> {
    let mut sections = Vec::new();
    for block in blocks {
        let instructions = generic_to_encodable_lines(block.lines);
        sections.push(PositionedEncodableBlock {
            addr: block.address,
            name: block.name,
            instructions,
        });
    }
    sections
}

// 2.12 Encoding blocks
//   Each line gets associated with some alignment

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

pub fn assemble(mut blocks: Vec<GenericBlock>) -> AssemblerTools {
    let metadata = extract_metadata(&mut blocks);

    let blocks = cast_generic_to_positioned_blocks(blocks);

    let blocks = gen_section_address(blocks, 0, 4);
    // println!("{:?}", blocks);
    // dbg!(&blocks);

    let blocks = gen_line_address(blocks);
    let blocks = gen_root_line_address(blocks);
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
