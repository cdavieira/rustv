use object::{
    ObjectSection,
    Endianness,
    Architecture,
    BinaryFormat,
    SectionKind,
};

use object::write::{
    self,
    SectionId,
};

use crate::spec::AssemblySectionName;



pub struct ElfWriter<'a> {
    obj: write::Object<'a>,
    text: SectionId,
    data: SectionId,
    bss: SectionId,
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
        ElfWriter { obj, text, data, bss, }
    }

    pub fn set_section_data(
        &mut self,
        section_name: AssemblySectionName,
        data: Vec<u8>,
        align: u64
    )
    {
        match section_name {
            AssemblySectionName::TEXT => {
                self.obj.section_mut(self.text).set_data(data, align);
            },
            AssemblySectionName::DATA => {
                self.obj.section_mut(self.data).set_data(data, align);
            },
            AssemblySectionName::BSS  => {
                // self.obj.section_mut(self.bss).set_data(data, align);
            },
            _ => {

            }
        }
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
        section_name: AssemblySectionName,
        rel_addr_to_sec_start: u64,
        name: &str,
        len: u64
    )
    {
        let (kind, section) = match section_name {
            AssemblySectionName::TEXT => {
                (write::SymbolKind::Label, self.text)
            },
            AssemblySectionName::DATA => {
                (write::SymbolKind::Data, self.data)
            },
            _ => panic!("Can't add symbol to this type of section"),
        };
        //the symbol is going to span from
        //<start of section + rel_addr>
        //to
        //<start of section + rel_addr + len>
        //OBS: linkage scope works like static scope in C and this is what we currently support
        self.obj.add_symbol(write::Symbol {
            name: name.bytes().collect(),
            value: rel_addr_to_sec_start,
            size: len, // ?
            kind,
            scope: write::SymbolScope::Linkage,
            section: write::SymbolSection::Section(section),
            weak: false,
            flags: write::SymbolFlags::None,
        });
    }

    pub fn save(&self, filename: &str) -> () {
        let elf_bytes = self.obj.write().expect("Failed loading bytes for save");
        std::fs::write(filename, elf_bytes).expect("Failed saving elf file");
    }
}
