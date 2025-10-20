use std::collections::HashMap;

use object::read::elf::ElfFile32;
use object::{
    self,
    Endianness,
    Object,
    ObjectSection,
    ObjectSymbol,
};
use object::read;

use crate::assembler::{self, AssemblerTools};
use crate::lang::highassembly::{self,};
use crate::streamreader::Position;
use crate::utils::swap_chunk_endianness;
use crate::lang::lowassembly::{self, DataEndianness};



// Error

#[derive(Debug)]
pub enum ElfReaderError {
    Parse(read::Error),
}

impl std::fmt::Display for ElfReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed when parsing elf file")
    }
}

impl From<read::Error> for ElfReaderError {
    fn from(value: read::Error) -> Self {
        ElfReaderError::Parse(value)
    }
}

pub type Result<T> = std::result::Result<T, ElfReaderError>;



// ElfReader

pub struct ElfSection {
    pub(crate) name: String,
    pub(crate) address: u32,
    pub(crate) align: usize,
    pub(crate) data: Vec<u8>,
}

pub struct ElfSymbol {
    pub(crate) name: String,
    pub(crate) address: u32,
    pub(crate) section: String,
    pub(crate) length: u64,
    pub(crate) scope: String,
}

pub struct ElfRelocation {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) offset: u64,
    pub(crate) addend: i32,
}

pub struct ElfReader<'a> {
    elf: ElfFile32<'a>,

    section_table: HashMap<String, ElfSection>,
    symbol_table: HashMap<String, ElfSymbol>,
    relocation_table: HashMap<String, ElfRelocation>,

    pc: usize,
}

impl<'a> ElfReader<'a> {
    pub fn new(
        data: &'a Vec<u8>,
        desired_endian: DataEndianness
    ) -> Result<ElfReader<'a>>
    {
        let elf = read::elf::ElfFile32::parse(data.as_slice())?;
        let section_table = build_section_table(&elf, &desired_endian);
        let symbol_table  = build_symbol_table(&elf);
        let relocation_table = build_relocation_table(&elf);
        let pc = if let Some(start) = elf.symbol_by_name("_start") {
            start.address() as usize
        }
        else {
            0
        };
        Ok(ElfReader {
            elf,
            section_table,
            symbol_table,
            relocation_table,
            pc,
        })
    }

    pub fn section(&self, name: &str) -> &ElfSection {
        self.section_table.get(name).unwrap()
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn tools(&self) -> AssemblerTools {
        let sections: HashMap<String, assembler::Section> = self.section_table
            .iter()
            .map(|pair| {
                let name = highassembly::SectionName::from_default_name(&pair.1.name);
                let section = assembler::Section {
                    address: pair.1.address as usize,
                    name,
                };
                (pair.1.name.to_string(), section)
            })
            .collect();
        let symbols: HashMap<String, assembler::Symbol> = self.symbol_table
            .iter()
            .map(|pair| {
                let section = highassembly::SectionName::from_default_name(&pair.1.section);
                let s = assembler::Symbol {
                    section,
                    relative_address: pair.1.address as usize,
                    length: pair.1.length as usize,
                    scope: pair.1.scope.clone(),
                };
                (pair.0.to_string(), s)
            })
            .collect();
        let mut relocations = HashMap::new();
        self.relocation_table
            .iter()
            .enumerate()
            .for_each(|(idx, pair)| {
                let rel = assembler::RelocationEntry {
                    id: idx,
                    address: pair.1.offset as usize,
                    addend: pair.1.addend
                };
                let relname = pair.0.clone();
                relocations
                    .entry(relname)
                    .or_insert(Vec::new())
                    .push(rel);
                // if relocations.contains_key(&relname) {
                //     relocations.insert(relname, vec![rel]);
                // }
                // else {
                //     let buffer = relocations.get_mut(&relname).unwrap();
                //     buffer.push(rel);
                // }
            });
        let blocks: Vec<_> = self.elf.sections()
            .map(|section| {
                let name = section.name().unwrap();
                let name = highassembly::SectionName::from_default_name(name);
                let addr = section.address() as usize;
                let data = section.data().unwrap();
                let data = DataEndianness::build_words_from_bytes(&data, DataEndianness::Le);
                let alignment = section.align() as usize;
                let instructions = data.into_iter().enumerate().map(|(idx, word)|
                    lowassembly::EncodedData {
                        data: vec![word],
                        alignment,
                        file_pos: Position::new(idx, idx, 0),
                    }
                ).collect();
                lowassembly::PositionedEncodedBlock {
                    addr,
                    name,
                    instructions,
                }
            })
            .collect();
        AssemblerTools {
            metadata: None,
            strings: Vec::new(),
            sections,
            symbols,
            relocations,
            blocks,
        }
    }
}

fn build_section_table<'a>(
    elf: &ElfFile32<'a>,
    desired_endian: &DataEndianness,
) -> HashMap<String, ElfSection> {
    let endian = match elf.endian() {
        Endianness::Little => &DataEndianness::Le,
        Endianness::Big => &DataEndianness::Be,
    };
    let mut section_table = HashMap::new();
    for section in elf.sections() {
        if let Ok(data) = section.data() {
            let align: usize = section.align().try_into().unwrap();
            let data: Vec<u8> = if endian == desired_endian {
                data.to_vec()
            } else { 
                if align > 1 {
                    swap_chunk_endianness(data, align)
                }
                else {
                    data.to_vec()
                }
            };
            let s = ElfSection {
                name: section.name().unwrap().to_string(),
                address: section.address() as u32,
                align,
                data,
            };
            section_table.insert(
                section.name().unwrap().to_string(),
                s
            );
        }
    }
    section_table
}

fn build_symbol_table<'a>(elf: &ElfFile32<'a>) -> HashMap<String, ElfSymbol> {
    let mut symbol_table = HashMap::new();
    for symbol in elf.symbols() {
        if let Ok(name) = symbol.name_bytes() {
            // kind, scope, flags, endian
            let name = String::from_utf8(name.to_vec()).unwrap();
            let address = symbol.address();
            let scope = match symbol.scope() {
                read::SymbolScope::Compilation => String::from("Compilation"),
                read::SymbolScope::Linkage => String::from("File"),
                read::SymbolScope::Dynamic => String::from("Dynamic"),
                read::SymbolScope::Unknown => panic!(),
            };
            let length = symbol.size() as u64;
            let section_symb = symbol.section();
            let section_idx = section_symb.index().unwrap();
            let section = elf.section_by_index(section_idx).unwrap();
            let s = ElfSymbol {
                name,
                address: address as u32,
                section: section.name().unwrap().to_string(),
                length,
                scope,
            };
            symbol_table.insert(
                s.name.clone(),
                s,
            );
        }
    }
    symbol_table
}

fn build_relocation_table<'a>(elf: &ElfFile32<'a>) -> HashMap<String, ElfRelocation> {
    let mut relocation_table = HashMap::new();
    let text_section = elf.section_by_name(".text").unwrap();
    let rel_sections = [text_section];
    for rel_section in rel_sections {
        for rel in rel_section.relocations() {
            let offset = rel.0;
            let addend = rel.1.addend();
            match rel.1.target() {
                read::RelocationTarget::Symbol(symbol_index) => {
                    let symbidx = symbol_index.0;
                    let symbol = elf.symbol_by_index(symbol_index).unwrap();
                    let name = symbol.name().unwrap().to_string();
                    let r = ElfRelocation {
                        id: symbidx,
                        name,
                        offset,
                        addend: addend as i32,
                    };
                    relocation_table.insert(
                        r.name.clone(),
                        r
                    );
                },
                read::RelocationTarget::Section(_section_index) => panic!(),
                read::RelocationTarget::Absolute => panic!(),
                _ => panic!(),
            }
        }
    }
    relocation_table
}
