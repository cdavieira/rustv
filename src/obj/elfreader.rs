use std::collections::HashMap;

use object::read::elf::ElfFile32;
use object::{
    self,
    Endianness,
    Object,
    ObjectSection,
};
use object::read;

use crate::utils::swap_chunk_endianness;
use crate::utils::DataEndianness;



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

pub struct ElfReader<'a> {
    elf: ElfFile32<'a>,

    section_table: HashMap<String, Vec<u8>>,
    // symbol_table: HashMap<String, Vec<u8>>,
}

impl<'a> ElfReader<'a> {
    pub fn new(
        data: &'a Vec<u8>,
        desired_endian: DataEndianness
    ) -> Result<ElfReader<'a>>
    {
        let elf = read::elf::ElfFile32::parse(data.as_slice())?;
        let section_table = build_section_table(&elf, &desired_endian);
        Ok(ElfReader {
            elf,
            section_table,
        })
    }

    pub fn text_section(&self) -> &Vec<u8> {
        self.section_table.get(".text").unwrap()
    }

    pub fn data_section(&self) -> &Vec<u8> {
        self.section_table.get(".data").unwrap()
    }
}

fn build_section_table<'a>(
    elf: &ElfFile32<'a>,
    desired_endian: &DataEndianness,
) -> HashMap<String, Vec<u8>> {
    let endian = match elf.endian() {
        Endianness::Little => &DataEndianness::Le,
        Endianness::Big => &DataEndianness::Be,
    };
    let mut section_table = HashMap::new();
    for section in elf.sections() {
        if let Ok(data) = section.data() {
            let data: Vec<u8> = if endian == desired_endian {
                data.to_vec()
            } else { 
                let align: usize = section.align().try_into().unwrap();
                if align > 1 {
                    swap_chunk_endianness(data, align)
                }
                else {
                    data.to_vec()
                }
            };
            section_table.insert(
                section.name().unwrap().to_string(),
                data
            );
        }
    }
    section_table
}

// fn build_symbol_table<'a>(elf: &ElfFile32<'a>) -> HashMap<String, u64> {
//     let mut symbol_table = HashMap::new();
//     for symbol in elf.symbols() {
//         if let Ok(name) = symbol.name_bytes() {
//             // kind, scope, flags, endian
//             let addr = symbol.address();
//             let _section = symbol.section();
//             let name = String::from_utf8(name.to_vec()).unwrap();
//             symbol_table.insert(
//                 name, addr
//             );
//         }
//     }
//     symbol_table
// }
