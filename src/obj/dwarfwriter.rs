use gimli::{write::{DebuggingInformationEntry, DwarfUnit}, LineProgram};
use object::write::SymbolId;

use crate::{assembler::AssemblerTools, lang::highassembly::SectionName, obj::elfwriter::ElfWriter, streamreader::Position};




/// Record information needed to write a section.
#[derive(Clone, Debug)]
struct DebugSection {
    data: gimli::write::EndianVec<gimli::LittleEndian>,
    relocations: Vec<gimli::write::Relocation>,
    id: Option<object::write::SectionId>,
}

impl DebugSection {
    fn new() -> Self {
        Self {
            data: gimli::write::EndianVec::new(gimli::LittleEndian),
            relocations: Vec::new(),
            id: None,
        }
    }
}

impl gimli::write::RelocateWriter for DebugSection {
    type Writer = gimli::write::EndianVec<gimli::LittleEndian>;

    fn writer(&self) -> &Self::Writer {
        &self.data
    }

    fn writer_mut(&mut self) -> &mut Self::Writer {
        &mut self.data
    }

    fn relocate(&mut self, relocation: gimli::write::Relocation) {
        self.relocations.push(relocation);
    }
}






const COMP_DIR: [u8; 2] = *b"./";

fn get_start<'a>(
    writer: & ElfWriter<'a>,
    start_name: &[u8],
) -> (SymbolId, u64)
{
    let _start_id   = writer.obj.symbol_id(start_name).unwrap();
    let _start_symb = writer.obj.symbol(_start_id);
    let text_section = writer.obj.section(writer.text);
    let textlen = text_section.data().len();
     (_start_id, textlen as u64)
}

fn add_die_root_info(
    dwarf: &mut DwarfUnit,
    root: gimli::write::UnitEntryId,
    file_name: &[u8],
    main_address: gimli::write::Address,
    range_list_id: gimli::write::RangeListId,
) -> ()
{
    let entry = dwarf.unit.get_mut(root);
    entry.set(
        gimli::DW_AT_producer,
        gimli::write::AttributeValue::String((*b"my assembly program").into()),
    );
    entry.set(gimli::DW_AT_name, gimli::write::AttributeValue::String(file_name.into()));
    entry.set(
        gimli::DW_AT_comp_dir,
        gimli::write::AttributeValue::String(COMP_DIR.into()),
    );
    entry.set(gimli::DW_AT_low_pc, gimli::write::AttributeValue::Address(main_address));
    entry.set(
        gimli::DW_AT_ranges,
        gimli::write::AttributeValue::RangeListRef(range_list_id),
    );
    // DW_AT_stmt_list will be set automatically.
}

fn add_die_main_function_info(
    dwarf: &mut DwarfUnit,
    root: gimli::write::UnitEntryId,
    main_name: &[u8],
    main_address: gimli::write::Address,
    main_size: u64,
    file_lines_id: gimli::write::FileId,
) -> ()
{
    let subprogram = dwarf.unit.add(root, gimli::DW_TAG_subprogram);
    let entry = dwarf.unit.get_mut(subprogram);
    entry.set(gimli::DW_AT_external, gimli::write::AttributeValue::Flag(true));
    entry.set(gimli::DW_AT_name, gimli::write::AttributeValue::String(main_name.into()));
    entry.set(
        gimli::DW_AT_decl_file,
        gimli::write::AttributeValue::FileIndex(Some(file_lines_id)),
    );
    entry.set(gimli::DW_AT_decl_line, gimli::write::AttributeValue::Udata(1));
    entry.set(gimli::DW_AT_decl_column, gimli::write::AttributeValue::Udata(1));
    entry.set(gimli::DW_AT_low_pc, gimli::write::AttributeValue::Address(main_address));
    entry.set(gimli::DW_AT_high_pc, gimli::write::AttributeValue::Udata(main_size as u64));
}

fn build_dwarf_sections<'a>(
    writer: &mut ElfWriter<'a>,
    dwarf: &mut DwarfUnit,
    main_symbol: SymbolId,
) -> ()
{
    let binary_format = object::BinaryFormat::native_object();

    // This will populate the sections with the DWARF data and relocations.
    let mut sections = gimli::write::Sections::new(DebugSection::new());
    dwarf.write(&mut sections).unwrap();

    // Add the DWARF section data to the object file.
    use gimli::write::Writer;
    sections.for_each_mut(|id, section| -> object::write::Result<()> {
        if section.data.len() == 0 {
            return Ok(());
        }
        let kind = if id.is_string() {
            object::SectionKind::DebugString
        } else {
            object::SectionKind::Debug
        };
        let section_id = writer.obj.add_section(Vec::new(), id.name().into(), kind);
        writer.obj.set_section_data(section_id, section.data.take(), 1);

        // Record the section ID so that it can be used for relocations.
        section.id = Some(section_id);
        Ok(())
    }).unwrap();

    // Add the relocations to the object file.
    sections.for_each(|_, section| -> object::write::Result<()> {
        let Some(section_id) = section.id else {
            debug_assert!(section.relocations.is_empty());
            return Ok(());
        };
        for reloc in &section.relocations {
            // println!("{:?}", reloc);
            // The `eh_pe` field is not used in this example because we are not writing
            // unwind information.
            debug_assert!(reloc.eh_pe.is_none());
            let (symbol, kind) = match reloc.target {
                gimli::write::RelocationTarget::Section(id) => {
                    let kind = if binary_format == object::BinaryFormat::Coff {
                        object::RelocationKind::SectionOffset
                    } else {
                        object::RelocationKind::Absolute
                    };
                    let symbol = writer.obj.section_symbol(sections.get(id).unwrap().id.unwrap());
                    (symbol, kind)
                }
                gimli::write::RelocationTarget::Symbol(id) => {
                    // The main function is the only symbol we have defined.
                    // debug_assert_eq!(id, 0);
                    // println!("{:?}", writer.obj.symbol(id));
                    // println!("{:?}", main_symbol);
                    // println!("{}", id);
                    (main_symbol, object::RelocationKind::Absolute)
                }
            };
            writer.obj.add_relocation(
                section_id,
                object::write::Relocation {
                    offset: reloc.offset as u64,
                    symbol,
                    addend: reloc.addend,
                    flags: object::RelocationFlags::Generic {
                        kind,
                        encoding: object::RelocationEncoding::Generic,
                        size: reloc.size * 8,
                    },
                },
            )?;
        }
        Ok(())
    }).unwrap();
}

fn add_line(
    line_program: &mut gimli::write::LineProgram,
    file_lines_id: gimli::write::FileId,
    line: u64,
    column: u64,
    address: u64,
)
{
    let row = line_program.row();
    row.address_offset = address;
    row.column = column;
    row.file = file_lines_id;
    row.line = line;
    line_program.generate_row();
}

pub fn add_debug_information<'a>(
    writer: &mut ElfWriter<'a>,
    tools: AssemblerTools,
    file_name: &[u8],
) -> () 
{
    let encoding = gimli::Encoding {
        format: gimli::Format::Dwarf32,
        version: 5,
        address_size: 4,
    };

    // Create a container for a single compilation unit.
    let mut dwarf = gimli::write::DwarfUnit::new(encoding);

    let root = dwarf.unit.root();

    // Create whats needed for later writing lines to the (dwarf section name???)
    // debugging section
    let line_strings = &mut dwarf.line_strings;

    let mut line_program = gimli::write::LineProgram::new(
        encoding,
        gimli::LineEncoding::default(),
        gimli::write::LineString::new(COMP_DIR, encoding, line_strings),
        None,
        gimli::write::LineString::new(file_name, encoding, line_strings),
        None,
    );

    let dir_id = line_program.default_directory();

    let file_lines = gimli::write::LineString::new(file_name, encoding, line_strings);

    let file_lines_id = line_program.add_file(file_lines, dir_id, None);




    let main_name = *b"_start";

    let (main_symbol, main_size) = get_start(writer, &main_name);

    let main_address = gimli::write::Address::Symbol {
        // This is a user defined identifier for the symbol.
        // In this case, we will use 0 to mean the main function.
        symbol: 0,
        addend: 0,
    };

    // Set attributes on the root DIE.
    let range_list_id = dwarf.unit.ranges.add(gimli::write::RangeList(vec![
        gimli::write::Range::StartLength {
            begin: main_address,
            length: main_size,
        },
    ]));

    add_die_root_info(&mut dwarf, root, &file_name, main_address, range_list_id);

    // Add program lines
    line_program.begin_sequence(Some(main_address));

    let sections: Vec<_> = tools.blocks.iter().filter(|block| block.name == SectionName::Text).collect();
    let textsection = sections.get(0).unwrap();
    let insts = textsection.instructions
        .iter()
        .rev()
        .skip(1)
        .rev();
    for (idx, inst) in insts.enumerate() {
        let row = (inst.file_pos.row() + 1) as u64;
        let col = inst.file_pos.col() as u64;
        add_line(
            &mut line_program,
            file_lines_id,
            row,
            col,
            (idx as u64)*4
        );
    }

    line_program.end_sequence(main_size as u64);

    // Commit program lines to the dwarf unit
    dwarf.unit.line_program = line_program;

    // Add a subprogram DIE for the main function.
    add_die_main_function_info(&mut dwarf, root, &main_name, main_address, main_size, file_lines_id);

    // Build the DWARF sections.
    build_dwarf_sections(writer, &mut dwarf, main_symbol);
}
