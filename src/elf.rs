use object::{
    self,
    Object,
    ObjectSection,
    Endianness,
    Architecture,
    BinaryFormat,
    SectionKind,
};
use object::write;
use object::read;
use std::collections::HashMap;

pub fn read_elf(filename: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>>{
    let mut instructions = Vec::new();

    // Opening the gates
    let data = std::fs::read(filename)?;
    let elf: read::elf::ElfFile32 = read::elf::ElfFile32::parse(data.as_slice())?;

    // Identifying sections
    // OBS: for now we only support 1 section of each type (.text, .data, .bss, ...)
    let mut sec_map = HashMap::new();
    for s in elf.sections() {
        if let Ok(d) = s.data() {
            sec_map.insert(
                s.name().unwrap().to_string(),
                d
            );
        }
    }

    // Retrieving the .text section
    // OBS: the following workaround has possibly something to do with the Endianness?
    let textsec = sec_map.get(".text").unwrap();
    for word in textsec.chunks(4) {
        let b0: u32 = word[0].into();
        let b1: u32 = word[1].into();
        let b2: u32 = word[2].into();
        let b3: u32 = word[3].into();
        let n = (b3 << 24) | (b2 << 16) | (b1 << 8) | b0;
        if n != 0 {
            instructions.push(n);
        }
        // else {
        //     eprintln!("read_elf: read null word");
        // }
    }

    Ok(instructions)
}

pub fn write_elf(filename: &str, v: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>{
    let mut obj = write::Object::new(BinaryFormat::Elf, Architecture::Riscv32, Endianness::Little);

    // let code_len = (&v).len();
    let section_data = vec![];
    let section_name = b".text".to_vec();
    let section_kind = SectionKind::Text;
    let text_section = obj.add_section(section_data, section_name, section_kind);

    // Add text section + instructions

    //NOTE:
    //For a instruction such as 'ecall' whose binary is '00000073',
    //'set_data' requires a vector such as 'vec![73, 0, 0, 0]'.

    //NOTE:
    //For two instructions such as 'ecall; ecall;',
    //the vector would be 'vec![73, 0, 0, 0, 73, 0, 0, 0]'

    // let machine_code: [u8; 4] = 0x00000073u32.to_le_bytes();
    // println!("{:?}", machine_code);
    // obj.section_mut(text_section).set_data(machine_code.to_vec(), 4);

    // obj.section_mut(text_section).set_data(v.into_iter().rev().collect::<Vec<u8>>(), 4);

    let mut u: Vec<u8> = Vec::new();
    for b in v.chunks(4) {
        u.push(b[3]);
        u.push(b[2]);
        u.push(b[1]);
        u.push(b[0]);
    }
    obj.section_mut(text_section).set_data(u, 4);

    // Add start symbol at the beginning of the text section
    let symb = write::Symbol {
        name: b"_start".to_vec(),
        value: 0,
        // size: code_len as u64,
        size: 0,
        kind: write::SymbolKind::Text,
        scope: write::SymbolScope::Linkage,
        section: write::SymbolSection::Section(text_section),
        weak: false,
        flags: write::SymbolFlags::None,
    };
    obj.add_symbol(symb);

    // Write to file
    let elf_bytes = obj.write().unwrap();
    std::fs::write(filename, elf_bytes)?;

    Ok(())
}
