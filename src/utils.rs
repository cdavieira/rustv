// General utilities
// use crate::debugger::SimpleGdbStub;
use crate::spec::{
    AssemblySectionName,
    AssemblyData,
};
use crate::tokenizer::Tokenizer;
use crate::lexer::Lexer;
// use crate::parser::Parser;
// use crate::assembler::Assembler;
// use crate::machine::{Machine, SimpleMachine};
use crate::syntax;
// use crate::elfwriter;
// use crate::elfreader;

// pub fn encode_to_words(code: &str) -> Vec<u32> {
//     let mut tokenizer = syntax::gas::Tokenizer;
//     let lexer = syntax::gas::Lexer;
//     let parser = syntax::gas::Parser;
//     let assembler = syntax::gas::Assembler;
//     let tokens = tokenizer.get_tokens(code);
//     // println!("{:?}", &tokens);
//     let lexemes = lexer.parse(tokens);
//     // println!("{:?}", &lexemes);
//     let mut parser_output = parser.parse(lexemes);
//     let text = parser_output.get_sections().into_iter().find(|stat| match stat.name {
//         AssemblySectionName::TEXT => true,
//         _ => false
//     }).unwrap();
//     assembler.to_words(text).data
// }
//
// pub fn encode_to_word(code: &str) -> u32 {
//     *encode_to_words(code).get(0).unwrap()
// }
//
// pub fn encode_to_elf(code: &str, output_file: &str) -> elfwriter::Result<()> {
//     let mut t = syntax::gas::Tokenizer;
//     let l = syntax::gas::Lexer;
//     let p = syntax::gas::Parser;
//     let s = syntax::gas::Assembler;
//
//     // Lexing
//     let tokens = t.get_tokens(code);
//     // println!("{:?}", &tokens);
//
//     let lexemes = l.parse(tokens);
//     // println!("{:?}", &lexemes);
//
//     // Obtaining parsing output
//     let parser_output = p.parse(lexemes);
//
//     let (_metadata, mut symbol_table, _section_table, sections) = parser_output.get_all();
//
//     let sections: Vec<AssemblyData> = sections
//         .into_iter()
//         .map(|section| {
//             s.to_words(section)
//         })
//         .collect();
//
//     // Writing to ELF
//     let mut writer = elfwriter::ElfWriter::new();
//     if symbol_table.contains_key("_start") {
//         let (_, start_symbol_addr) = *symbol_table.get("_start").unwrap();
//         writer.set_start_address(start_symbol_addr.try_into().unwrap());
//         let _ = symbol_table.remove("_start").unwrap();
//     }
//     else {
//         writer.set_start_address(0);
//     }
//
//     // TODO: handle what length actually means
//     for (symb, (sect_name, rel_addr)) in symbol_table {
//         let length = 0;
//         writer.add_symbol(sect_name, rel_addr.try_into().unwrap(), &symb, length);
//     }
//     for section in sections {
//         if section.data.len() > 0 {
//             let name = section.name;
//             let data = words_to_bytes_le(&section.data);
//
//             //TODO: THIS ALIGNMENT IS WRONG, I ONLY DID THIS TO TEST ONE THING
//             match name {
//                 AssemblySectionName::TEXT => {
//                     writer.set_section_data(name, data, 4);
//                 },
//                 AssemblySectionName::DATA => {
//                     writer.set_section_data(name, data, 1);
//                 },
//                 _ => {}
//             }
//         }
//     }
//
//     writer.save(output_file)
// }
//
// pub fn new_machine_from_elf_textsection(filename: &str) -> SimpleMachine {
//     let data = std::fs::read(filename)
//         .expect("Failed reading elf file");
//
//     let reader = elfreader::ElfReader::new(
//         &data,
//         DataEndianness::LE
//     )
//         .expect("Failed instantiating elf file reader");
//
//     let textdata = reader.text_section();
//     print_bytes_hex(textdata);
//
//     SimpleMachine::from_bytes(textdata, DataEndianness::BE)
// }
//
// pub fn new_machine_from_bytes(text_bytes: &Vec<u8>) -> SimpleMachine {
//     // print_bytes_hex(text_bytes);
//     SimpleMachine::from_bytes(text_bytes, DataEndianness::BE)
// }
//
// pub fn new_machine_from_words(text_words: &Vec<u32>) -> SimpleMachine {
//     // print_bytes_hex(text_bytes);
//     SimpleMachine::from_words(text_words, DataEndianness::BE)
// }
//
// pub fn wait_for_new_debugger_at_port<'a>(memsize: usize, port: u16) -> SimpleGdbStub<'a, SimpleMachine> {
//     SimpleGdbStub::<SimpleMachine>::new(memsize, port)
//         .expect("Failed when instantiating riscv32 debugger")
// }




// Data conversion

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

/// Performs a right shift of 'bit_idx', followed by applying a 1bit-mask to all remaining
/// 'bit_amount' bits
///
/// Index convention (with the number 1 as an example):
///   (1 in binary) ->    00000000     00000001     00000000   00000001
///   ( bit index ) ->   31      24   23      16   15      8   7      0
///
/// Boundaries:
/// bit_idx: 0-31
/// bit_amount: 1-32
///
/// Example 1: obtain the 4th bit until the 7th bit of a number
/// mask_lower_bits(n, 3, 4)
///
/// Example 2: obtain the first 12 bits of a number
/// mask_lower_bits(n, 0, 12)
pub fn rsh_mask_bits(n: &u32, bit_idx: u8, bit_amount: usize) -> u32 {
    (n >> bit_idx) & UWORD_MASK[bit_amount]
}

/// Converts a vector of u32 into a vector of u8, ensuring Big Endianness for the resulting bytes
/// in the process
pub fn words_to_bytes_be(words: &Vec<u32>) -> Vec<u8> {
    //n.to_be_bytes() = [n[24..32], n[16..24], n[8..16], n[0..8]]
    words
        .iter()
        .map(|word| u32::to_be_bytes(*word))
        .flatten()
        .collect()
}

/// Converts a vector of u32 into a vector of u8, ensuring Little Endianness for the resulting
/// bytes in the process
pub fn words_to_bytes_le(words: &Vec<u32>) -> Vec<u8> {
    //n.to_le_bytes() = [n[0..8], n[8..16], n[16..24], n[24..32]]
    words
        .iter()
        .map(|word| u32::to_le_bytes(*word))
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







// Other

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DataEndianness {
    LE,
    BE,
}

impl DataEndianness {
    pub fn from_bytes_to_word(&self, bytes: [u8; 4]) -> u32 {
        match self {
            DataEndianness::LE => u32::from_le_bytes(bytes),
            DataEndianness::BE => u32::from_be_bytes(bytes),
        }
    }

    pub fn from_word_to_bytes(&self, word: u32) -> [u8; 4] {
        match self {
            DataEndianness::LE => u32::to_le_bytes(word),
            DataEndianness::BE => u32::to_be_bytes(word),
        }
    }

    pub fn change_endian_word_to_word(&self, n: u32, target: DataEndianness) -> u32 {
        match self {
            DataEndianness::LE => {
                if target == DataEndianness::LE {
                    n
                }
                else {
                    u32::to_le(n)
                }
            },
            DataEndianness::BE => {
                if target == DataEndianness::BE {
                    n
                }
                else {
                    u32::to_be(n)
                }
            },
        }
    }

    pub fn change_endian_bytes_to_word(&self, bytes: [u8; 4], target: DataEndianness) -> u32 {
        match self {
            DataEndianness::LE => {
                let val = u32::from_le_bytes(bytes);
                if target == DataEndianness::LE {
                    val
                }
                else {
                    u32::to_be(val)
                }
            },
            DataEndianness::BE => {
                let val = u32::from_be_bytes(bytes);
                if target == DataEndianness::BE {
                    val
                }
                else {
                    u32::to_le(val)
                }
            },
        }
    }
}
