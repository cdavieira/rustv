use object::elf::{
    R_RISCV_PCREL_HI20,
    R_RISCV_PCREL_LO12_I,
};
use object::{
    Architecture,
    BinaryFormat,
    Endianness,
    ObjectSection,
    SectionKind,
    // RelocationTarget,
    // SymbolSection
};

use object::write::{
    self,
    SectionId,
    SymbolId,
    Relocation,
    // RelocationKind,
};

use std::collections::hash_map::HashMap;

use crate::lang::highassembly::SectionName;






// Result

#[derive(Debug)]
pub enum ElfWriterError {
    Build(write::Error),
    IO(std::io::Error),
    WrongSection(String)
}

impl std::fmt::Display for ElfWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "elf writter failed!")
    }
}

impl From<write::Error> for ElfWriterError {
    fn from(value: write::Error) -> Self {
        ElfWriterError::Build(value)
    }
}

impl From<std::io::Error> for ElfWriterError {
    fn from(value: std::io::Error) -> Self {
        ElfWriterError::IO(value)
    }
}

pub type Result<T> = std::result::Result<T, ElfWriterError>;





// ElfWriter

pub struct ElfWriter<'a> {
    obj: write::Object<'a>,
    text: SectionId,
    data: SectionId,
    bss: SectionId,
    symbol_ids: HashMap<String, SymbolId>,
}

impl<'a> ElfWriter<'a> {
    pub fn new() -> Self {
        let mut obj = write::Object::new(
            BinaryFormat::Elf,
            Architecture::Riscv32,
            Endianness::Little
        );
        let text = obj.add_section(Vec::new(), b".text".to_vec(), SectionKind::Text);
        let data = obj.add_section(Vec::new(), b".data".to_vec(), SectionKind::Data);
        let bss  = obj.add_section(Vec::new(), b".bss".to_vec(), SectionKind::UninitializedData);
        let symbol_ids = HashMap::new();
        ElfWriter { obj, text, data, bss, symbol_ids, }
    }

    pub fn set_section_data(
        &mut self,
        section_name: SectionName,
        data: Vec<u8>,
        align: u64
    ) -> Result<()>
    {
        let secid = match section_name {
            SectionName::Text => self.text,
            SectionName::Data => self.data,
            SectionName::Bss  => self.bss,
            _ => return Err(ElfWriterError::WrongSection(String::from("Can't data to custom sections yet")))
        };
        self.obj.section_mut(secid).set_data(data, align);
        Ok(())
    }

    pub fn set_start_address(&mut self, rel_addr_to_text_sec: u64) {
        self.obj.add_symbol(write::Symbol {
            name: b"_start".to_vec(),
            value: rel_addr_to_text_sec,
            // size: code_len as u64,
            size: 0,
            kind: write::SymbolKind::Text,
            scope: write::SymbolScope::Linkage,
            section: write::SymbolSection::Section(self.text),
            weak: false,
            flags: write::SymbolFlags::None,
        });
    }

    pub fn add_symbol(
        &mut self,
        section_name: SectionName,
        rel_addr_to_sec_start: u64,
        name: &str,
        len: u64
    )
    {
        let (kind, section) = match section_name {
            SectionName::Text => {
                (write::SymbolKind::Label, self.text)
            },
            SectionName::Data => {
                (write::SymbolKind::Data, self.data)
            },
            _ => panic!("Can't add symbol to this type of section"),
        };

        //the symbol is going to span from
        //<start of section + rel_addr>
        //to
        //<start of section + rel_addr + len>
        //OBS: linkage scope works like static scope in C and this is what we currently support
        let symbol_id = self.obj.add_symbol(write::Symbol {
            name: name.bytes().collect(),
            value: rel_addr_to_sec_start,
            size: len, // ?
            kind,
            scope: write::SymbolScope::Linkage,
            section: write::SymbolSection::Section(section),
            weak: false,
            flags: write::SymbolFlags::None,
        });

        self.symbol_ids.insert(name.to_string(), symbol_id);
    }

    pub fn handle_symbol_relocation(
        &mut self,
        symbol_name: &str,
        text_section_off: u64,
        symbol_addend: i32,
        idx: usize,
    ) -> Result<()>
    {
        create_ext_symbol_relocatable_reference(
            &mut self.obj,
            self.text,
            text_section_off,
            &self.symbol_ids,
            symbol_name,
            symbol_addend,
            idx
        )
    }

    pub fn save(&self, filename: &str) -> Result<()> {
        let elf_bytes = self.obj.write()?;
        std::fs::write(filename, elf_bytes)?;
        Ok(())
    }
}


fn create_ext_symbol_relocatable_reference<'a>(
    obj: &mut write::Object<'a>,
    text_section_id: SectionId,
    text_section_off: u64,
    symbol_ids: &HashMap<String, SymbolId>,
    symbol_name: &str,
    symbol_addend: i32,
    tmpidx: usize,
) -> Result<()>
{
    let symbol_id = symbol_ids
        .get(symbol_name)
        .expect("Symbol id not found when creating relocation");
    let hi_off = text_section_off;
    let lo_off = text_section_off + 4;

    let tmp_label_name = String::from(".L") + symbol_name + &tmpidx.to_string();

    let tmp_label = write::Symbol {
        name: tmp_label_name.bytes().collect(),
        //4 bytes, if this label refers to an opcode instruction
        value: hi_off,
        size: 0,
        kind: object::SymbolKind::Text,
        //this label is only needed during the compilation, no need to expose it in the resulting
        //ELF file
        scope: write::SymbolScope::Compilation,
        weak: false,
        section: object::write::SymbolSection::Section(text_section_id),
        flags: object::SymbolFlags::None,
    };

    let tmp_label_id = obj.add_symbol(tmp_label);

    let symbol_hi_relocation = Relocation {
        offset: hi_off,
        symbol: *symbol_id,
        addend: symbol_addend as i64,
        flags: write::RelocationFlags::Elf { r_type: R_RISCV_PCREL_HI20 },
    };

    let symbol_lo_relocation = Relocation {
        offset: lo_off,
        symbol: tmp_label_id,
        addend: symbol_addend as i64,
        flags: write::RelocationFlags::Elf { r_type: R_RISCV_PCREL_LO12_I },
    };

    obj.add_relocation(text_section_id, symbol_hi_relocation)
        .expect("Failed adding Hi relocation");

    obj.add_relocation(text_section_id, symbol_lo_relocation)
        .expect("Failed adding Lo relocation");

    Ok(())
}
