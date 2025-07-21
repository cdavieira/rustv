  use object::{self, Object, Endianness, Architecture, BinaryFormat, SectionKind};
  use object::write;
  
  pub fn write_elf(filename: &str, v: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>{
      let mut obj = write::Object::new(BinaryFormat::Elf, Architecture::Riscv32, Endianness::Little);
      let text_section = obj.add_section(vec![], b".text".to_vec(), SectionKind::Text);
  
      // Set contents
      obj.section_mut(text_section).set_data(v, 4);
  
      // Write to file
      let elf_bytes = obj.write().unwrap();
      std::fs::write(filename, elf_bytes)?;
  
      Ok(())
  }
