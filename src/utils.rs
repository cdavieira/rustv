// General utilities
use crate::cpu::{SimpleCPU, CPU};
use crate::memory::{SimpleMemory, Memory};
use crate::spec::{
    AssemblySectionName,
    AssemblyData,
};
use crate::tokenizer::Tokenizer;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::assembler::Assembler;
use crate::machine::Machine;
use crate::syntax;
use crate::elfwriter;

// TODO: create helper function to formalize the routine of reading from an elf file and executing
// TODO: create helper function to formalize the routine of executing the debugger
// TODO: create helper function to formalize the routine of executing from a bare vector of words

pub fn encode_to_words(code: &str) -> Vec<u32> {
    let mut tokenizer = syntax::gas::Tokenizer;
    let lexer = syntax::gas::Lexer;
    let parser = syntax::gas::Parser;
    let assembler = syntax::gas::Assembler;
    let tokens = tokenizer.get_tokens(code);
    let lexemes = lexer.parse(tokens);
    let mut parser_output = parser.parse(lexemes);
    let text = parser_output.get_sections().into_iter().find(|stat| match stat.name {
        AssemblySectionName::TEXT => true,
        _ => false
    }).unwrap();
    assembler.to_words(text).data
}

pub fn encode_to_word(code: &str) -> u32 {
    *encode_to_words(code).get(0).unwrap()
}

pub fn encode_to_elf(code: &str, output_file: &str) -> () {
    let mut t = syntax::gas::Tokenizer;
    let l = syntax::gas::Lexer;
    let p = syntax::gas::Parser;
    let s = syntax::gas::Assembler;

    // Lexing
    let tokens = t.get_tokens(code);
    // println!("{:?}", &tokens);

    let lexemes = l.parse(tokens);
    // println!("{:?}", &lexemes);

    // Obtaining parsing output
    let parser_output = p.parse(lexemes);

    let (metadata, mut symbol_table, section_table, sections) = parser_output.get_all();

    let sections: Vec<AssemblyData> = sections
        .into_iter()
        .map(|section| {
            s.to_words(section)
        })
        .collect();

    // Writing to ELF
    let mut writer = elfwriter::ElfWriter::new();
    if symbol_table.contains_key("_start") {
        let (_, start_symbol_addr) = *symbol_table.get("_start").unwrap();
        writer.set_start_address(start_symbol_addr.try_into().unwrap());
        let _ = symbol_table.remove("_start").unwrap();
    }
    else {
        writer.set_start_address(0);
    }

    // TODO: handle what length actually means
    for (symb, (sect_name, rel_addr)) in symbol_table {
        let length = 0;
        writer.add_symbol(sect_name, rel_addr.try_into().unwrap(), &symb, length);
    }
    for section in sections {
        if section.data.len() > 0 {
            let name = section.name;
            // let data = swap_words_endianness(uwords_to_bytes(&section.data));

            let data = swap_words_endianness(uwords_to_bytes(&section.data));

            //TODO: THIS ALIGNMENT IS WRONG, I ONLY DID THIS TO TEST ONE THING
            match name {
                AssemblySectionName::TEXT => {
                    writer.set_section_data(name, data, 4);
                },
                AssemblySectionName::DATA => {
                    writer.set_section_data(name, data, 1);
                },
                _ => {}
            }
        }
    }

    writer.save(output_file);
}

fn uwords_to_bytes(uwords: &Vec<u32>) -> Vec<u8> {
    let mut v = Vec::new();
    for uword in uwords {
        let bytes: [u8; 4] = u32::to_be_bytes(*uword);
        v.push(bytes[3]);
        v.push(bytes[2]);
        v.push(bytes[1]);
        v.push(bytes[0]);
    }
    v
}

fn swap_words_endianness(v: Vec<u8>) -> Vec<u8> {
    let mut u: Vec<u8> = Vec::new();
    for b in v.chunks_exact(4) {
        u.push(b[3]);
        u.push(b[2]);
        u.push(b[1]);
        u.push(b[0]);
    }
    u
}




/// Retrieves a mask to be used with the '&' to filter the first <n> bits of a word
/// For example:
///   n = 0b11101
///   Obtaining the first 3 bits of 'n':
///   first_3_bits = n & UWORD_MASK[3]
///   first_3_bits = n & 0b111
///   first_3_bits == 0b00101
const UWORD_MASK: [u32; 33] = [
    0b0, //Not used
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

/// Performs a right shift of 'bit_idx', followed by applying a 1bit-mask to all remaining 'bit_amount' bits
///
/// Index convention (with the number 1 as an example):
///   (binary) ->  0 ... 00000000 00000001
///   (index ) -> 31 ...15      8 7      0
///
/// Boundaries:
/// 0 lte bit_idx lt 32
/// 1 lte bit_amount lte 32
///
/// Example 1: obtain the 4th bit until the 7th bit of a number
/// mask_lower_bits(n, 4, 4)
///
/// Example 2: obtain the first 12 bits of a number
/// mask_lower_bits(n, 0, 12)
pub fn rsh_mask_bits(n: &u32, bit_idx: u8, bit_amount: usize) -> u32 {
    (n >> bit_idx) & UWORD_MASK[bit_amount]
}
