use crate::assembler::{Assembler, AssemblerTools};
use crate::emu::debugger::SimpleGdbStub;
use crate::emu::machine::{Machine, SimpleMachine};
use crate::lang::highassembly::SectionName;
use crate::lang::lowassembly::{DataEndianness, EncodedData};
use crate::lexer::Lexer;
use crate::obj::dwarfwriter::add_debug_information;
use crate::obj::elfreader;
use crate::obj::elfwriter;
use crate::parser::Parser;
use crate::syntax;
use crate::tokenizer::Tokenizer;

pub fn build_code_repr(code: &str) -> AssemblerTools {
    let mut lexer = syntax::gas::Lexer;
    let tokenizer = syntax::gas::Tokenizer;
    let parser = syntax::gas::Parser;
    let assembler = syntax::gas::Assembler;

    let lexemes = lexer.get_tokens(code);
    // println!("{:?}", tokens);
    // dbg!(&tokens);

    let tokens = tokenizer.parse(lexemes);
    // println!("{:?}", lexemes);
    // dbg!(&lexemes);

    let blocks = parser.parse(tokens);
    // dbg!(&blocks);

    let tools = assembler.assemble(blocks);
    // dbg!(&tools);

    tools
}

pub fn encode_to_words(code: &str) -> Vec<u32> {
    build_code_repr(code).text_section_words()
}

pub fn encode_to_word(code: &str) -> u32 {
    *encode_to_words(code).get(0).unwrap()
}

fn write_from_tools<'a>(mut output: AssemblerTools) -> (elfwriter::ElfWriter<'a>, AssemblerTools) {
    let mut writer = elfwriter::ElfWriter::new();
    let symbol_table = &mut output.symbols;
    let relocation_table = &output.relocations;
    let blocks = &output.blocks;

    if symbol_table.contains_key("_start") {
        let symb = symbol_table.get("_start").expect("No _start found");
        writer.set_start_address(symb.relative_address.try_into().unwrap());
        symbol_table.remove("_start").unwrap();
    } else {
        writer.set_start_address(0);
    }

    for (name, symb) in symbol_table {
        let symbol_section = symb.section.clone();
        let symbol_addr = symb.relative_address.try_into().unwrap();
        let length = symb.length;
        writer.add_symbol(symbol_section, symbol_addr, &name, length as u64);
    }

    for block in blocks {
        if block.instructions.len() > 0 {
            let name = &block.name;
            let data = encoded_data_to_bytes_le(&block.instructions);
            let alignment = match name {
                SectionName::Text => 4,
                SectionName::Data => 1,
                _ => panic!(""),
            };
            writer
                .set_section_data(name.clone(), data, alignment)
                .expect("");
        }
    }

    for (symbname, relocations) in relocation_table {
        for relocation in relocations {
            let offset = relocation.address.try_into().unwrap();
            let addend = relocation.addend;
            let relidx = relocation.id;
            writer
                .handle_symbol_relocation(symbname, offset, addend, relidx)
                .unwrap();
        }
    }

    (writer, output)
}

pub fn encode_to_elf(code: &str, output_file: &str) -> elfwriter::Result<()> {
    let output = build_code_repr(code);
    let (writer, _) = write_from_tools(output);
    writer.save(output_file)
}

pub fn encode_to_elf_with_debug(
    code: &str,
    input_file: &str,
    output_file: &str,
) -> elfwriter::Result<()> {
    let output = build_code_repr(code);
    let (mut writer, tools) = write_from_tools(output);
    add_debug_information(&mut writer, tools, input_file.as_bytes());
    writer.save(output_file)
}

pub fn new_machine_from_tools(tools: &AssemblerTools) -> SimpleMachine {
    let text_start = tools.text_section_start();
    let textdata = tools.text_section_bytes_be();

    let data_start = tools.data_section_start();
    let datadata = tools.data_section_bytes_be();

    let minsize = textdata.len() + datadata.len();
    let max_start = if text_start > data_start {
        text_start
    } else {
        data_start
    };
    let memsize = if max_start > minsize {
        max_start + minsize
    } else {
        max_start + (minsize - max_start)
    } + 4usize;

    let pc = 0;

    let mut m = SimpleMachine::from_bytes_size(memsize, DataEndianness::Be);
    m.write_memory_bytes(text_start, &textdata);
    m.write_memory_bytes(data_start, &datadata);
    m.jump(pc);
    m
}

pub fn new_machine_from_elf(filename: &str) -> SimpleMachine {
    let data = std::fs::read(filename).expect("Failed reading elf file");

    let reader = elfreader::ElfReader::new(&data, DataEndianness::Be)
        .expect("Failed instantiating elf file reader");

    let textsec = reader.section(".text").unwrap();
    let datasec = reader.section(".data");

    let textdata = &textsec.data;
    let text_start = textsec.address as usize;

    let has_data_section = datasec.is_some();

    let (memsize, data_start, datadata) = {
        if !has_data_section {
            let memsize = text_start + textdata.len();
            (memsize, None, None)
        } else {
            let datadata = &datasec.unwrap().data;
            let data_start = datasec.unwrap().address as usize;
            let minsize = textdata.len() + datadata.len();
            let max_start = if text_start > data_start {
                text_start
            } else {
                data_start
            };
            let memsize = if max_start > minsize {
                max_start + minsize
            } else {
                max_start + (minsize - max_start)
            };
            (memsize, Some(data_start), Some(datadata))
        }
    };

    let pc = reader.pc();

    let mut m = SimpleMachine::from_bytes_size(memsize, DataEndianness::Be);

    m.write_memory_bytes(text_start, textdata);
    if has_data_section {
        m.write_memory_bytes(data_start.unwrap(), datadata.unwrap());
    }
    m.jump(pc);

    m
}

pub fn new_machine_from_bytes(text_bytes: &Vec<u8>) -> SimpleMachine {
    // print_bytes_hex(text_bytes);
    SimpleMachine::from_bytes(text_bytes, DataEndianness::Be)
}

pub fn new_machine_from_words(text_words: &Vec<u32>) -> SimpleMachine {
    // print_bytes_hex(text_bytes);
    SimpleMachine::from_words(text_words, DataEndianness::Be)
}

pub fn wait_for_new_debugger_at_port<'a>(
    memsize: usize,
    port: u16,
) -> SimpleGdbStub<'a, SimpleMachine> {
    SimpleGdbStub::<SimpleMachine>::new(memsize, port)
        .expect("Failed when instantiating riscv32 debugger")
}

// Data conversion

/// Retrieves a mask to be used with the '&' to filter the first <n> bits of a word
/// For example:
///   n = 0b11101
///   Obtaining the first 3 bits of 'n':
///   `first_3_bits = n & UWORD_MASK[3]`
///   `first_3_bits = n & 0b111`
///   `first_3_bits == 0b00101`
const UWORD_MASK: [u32; 33] = [
    0b0,
    0b1,
    0b11,
    0b111,
    0b1111,
    0b11111,
    0b111111,
    0b1111111,
    0b11111111,
    0b111111111,
    0b1111111111,
    0b11111111111,
    0b111111111111,
    0b1111111111111,
    0b11111111111111,
    0b111111111111111,
    0b1111111111111111,
    0b11111111111111111,
    0b111111111111111111,
    0b1111111111111111111,
    0b11111111111111111111,
    0b111111111111111111111,
    0b1111111111111111111111,
    0b11111111111111111111111,
    0b111111111111111111111111,
    0b1111111111111111111111111,
    0b11111111111111111111111111,
    0b111111111111111111111111111,
    0b1111111111111111111111111111,
    0b11111111111111111111111111111,
    0b111111111111111111111111111111,
    0b1111111111111111111111111111111,
    0b11111111111111111111111111111111,
];

/// Performs a right shift of 'bit_idx', followed by applying a 1bit-mask to all remaining
/// 'bit_amount' bits
///
/// This operation corresponds to obtaining a number of 'n' bits starting at index 'bit_idx'
///
/// Index convention (with the number 1 as an example):
///   (1 in binary) ->    00000000     00000000     00000000   00000001
///   ( bit index ) ->   31      24   23      16   15      8   7      0
///
/// Boundaries:
/// bit_idx: 0-31
/// bit_amount: 1-32
///
/// Example 1: obtain the 4th bit until the 7th bit of a number
/// `mask_lower_bits(n, 3, 4)`
/// which roughly translates to
/// `(n >> 3) & 0b1111`
///
pub fn get_n_bits_from(n: &u32, bit_idx: u8, bit_amount: usize) -> u32 {
    (n >> bit_idx) & UWORD_MASK[bit_amount]
}

/// Example: get_bits_from_to(0b1111, 1, 2) -> 0b0110
///
/// Index convention (with the number 1 as an example):
///   (1 in binary) ->    00000000     00000000     00000000   00000001
///   ( bit index ) ->   31      24   23      16   15      8   7      0
pub fn get_bits_range(n: u32, start: usize, end: usize) -> u32 {
    let mask = UWORD_MASK[end + 1] & (!UWORD_MASK[start]);
    (n & mask) >> start
}

/// Index convention (with the number 1 as an example):
///   (1 in binary) ->    00000000     00000000     00000000   00000001
///   ( bit index ) ->   31      24   23      16   15      8   7      0
pub fn get_bit_at(n: u32, idx: usize) -> u32 {
    (n >> idx) & 0b1
}

/// Sets all bits to the left of 'start' to be the same value as 'bit'
///
/// Example: set_remaining_bits(0b0011, 3, 1) -> 0b1..10011
///
/// Index convention (with the number 1 as an example):
///   (1 in binary) ->    00000000     00000000     00000000   00000001
///   ( bit index ) ->   31      24   23      16   15      8   7      0
pub fn set_remaining_bits(n: u32, start: usize, bit: usize) -> u32 {
    let mask = UWORD_MASK[start];
    if bit == 0 { n & mask } else { n | !mask }
}

/// Converts a vector of u32 into a vector of u8, ensuring Big Endianness for the resulting bytes
pub fn words_to_bytes_be(words: &Vec<u32>) -> Vec<u8> {
    //n.to_be_bytes() = [n[24..32], n[16..24], n[8..16], n[0..8]]
    words
        .iter()
        .map(|word| u32::to_be_bytes(*word))
        .flatten()
        .collect()
}

/// Converts a vector of u32 into a vector of u8, ensuring Little Endianness for the resulting
/// bytes
pub fn words_to_bytes_le(words: &Vec<u32>) -> Vec<u8> {
    //n.to_le_bytes() = [n[0..8], n[8..16], n[16..24], n[24..32]]
    words
        .iter()
        .map(|word| u32::to_le_bytes(*word))
        .flatten()
        .collect()
}

pub fn encoded_data_to_bytes_le(data: &Vec<EncodedData>) -> Vec<u8> {
    data.into_iter()
        .map(|words_data| {
            let mut raw_bytes = Vec::new();
            for word in &words_data.data {
                let word_bytes = u32::to_le_bytes(*word);
                raw_bytes.extend(&word_bytes);
            }
            raw_bytes
        })
        .flatten()
        .collect()
}

/// Iterates chunks of 'chunk_size' bytes at a time and swaps the endianness of the bytes within
/// that chunk
pub fn swap_chunk_endianness(v: &[u8], chunk_size: usize) -> Vec<u8> {
    let mut u: Vec<u8> = Vec::new();
    for b in v.chunks_exact(chunk_size) {
        u.push(b[3]);
        u.push(b[2]);
        u.push(b[1]);
        u.push(b[0]);
    }
    u
}

// Data visualization

pub fn print_words_hex(data: &[u32]) {
    for word in data {
        print!("{:02X} ", word);
    }
    println!();
}

pub fn print_bytes_hex(data: &[u8]) {
    for byte in data {
        print!("{:02X} ", byte);
    }
    println!();
}

pub fn print_binary_int(n: u32) {
    println!("{:032b}", n);
}
