// Low Assembly Types are types which handle:
// * symbol resolution
// * absolute/relative address assignment
// * low level details (such as binary encoding/representation)

use crate::{
    lang::{
        directive::Directive,
        ext::Extension,
        highassembly::SectionName,
    },
};

use super::ext::instruction_to_binary;




#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DataEndianness {
    Le,
    Be,
}

impl DataEndianness {
    pub fn induce_bytes_to_words(bytes: &[u8], target: DataEndianness) -> Vec<u32> {
        let callback = match target {
            DataEndianness::Le => u32::from_le_bytes,
            DataEndianness::Be => u32::from_be_bytes,
        };
        bytes
            .chunks(4)
            .map(|chunk| {
                let word_bytes: [u8; 4] = chunk
                    .try_into()
                    .expect("Error encoding data for directive");
                callback(word_bytes)
            })
            .collect()
    }

    pub fn induce_bytes_to_word(bytes: [u8; 4], target: DataEndianness) -> u32 {
        match target {
            DataEndianness::Le => u32::from_le_bytes(bytes),
            DataEndianness::Be => u32::from_be_bytes(bytes),
        }
    }

    pub fn induce_word_to_bytes(word: u32, target: DataEndianness) -> [u8; 4] {
        match target {
            DataEndianness::Le => u32::to_le_bytes(word),
            DataEndianness::Be => u32::to_be_bytes(word),
        }
    }

    pub fn modify_word_to_word(n: u32, source: DataEndianness, target: DataEndianness) -> u32 {
        match source {
            DataEndianness::Le => {
                if target == DataEndianness::Le {
                    n
                }
                else {
                    u32::to_le(n)
                }
            },
            DataEndianness::Be => {
                if target == DataEndianness::Be {
                    n
                }
                else {
                    u32::to_be(n)
                }
            },
        }
    }

    pub fn modify_bytes_to_word(bytes: [u8; 4], source: DataEndianness, target: DataEndianness) -> u32 {
        match source {
            DataEndianness::Le => {
                let val = u32::from_le_bytes(bytes);
                if target == DataEndianness::Le {
                    val
                }
                else {
                    u32::to_be(val)
                }
            },
            DataEndianness::Be => {
                let val = u32::from_be_bytes(bytes);
                if target == DataEndianness::Be {
                    val
                }
                else {
                    u32::to_le(val)
                }
            },
        }
    }
}




#[derive(Debug)]
pub struct EncodedData {
    pub data: Vec<u32>,
    pub alignment: usize,
}





#[derive(Debug)]
pub enum EncodableKey {
    Op(Box<dyn Extension>),
    Directive(Box<dyn Directive>),
}

#[derive(Debug)]
pub struct EncodableLine {
    pub key: EncodableKey,
    pub args: Vec<i32>,
}

impl EncodableLine {
    pub fn encode(self) -> EncodedData {
        match self.key {
            EncodableKey::Op(op) => {
                let data = vec![instruction_to_binary(&op, &self.args)];
                EncodedData {
                    data,
                    alignment: 4,
                }
            },
            EncodableKey::Directive(d) => {
                let alignment = d.datatype().alignment();
                // println!("{:?}", &self.args);
                let data: Vec<u32> = {
                    let len_args = self.args.len();
                    let exceeding_bytes = len_args % 4; //for word boundary
                    let pad = if exceeding_bytes > 0 {
                        4 - exceeding_bytes
                    } else {
                        0
                    };
                    let mut args = self.args.clone();
                    for _ in 0..pad {
                        args.push(0);
                    }
                    let args: Vec<u8> = args
                        .into_iter()
                        .map(|arg| arg as u8)
                        .collect();
                    DataEndianness::induce_bytes_to_words(&args, DataEndianness::Le)
                };
                // print_words_hex(&data[..]);
                EncodedData {
                    data,
                    alignment,
                }
            },
        }
    }
}





pub struct PositionedEncodableLine {
    pub addr: usize,
    pub line: EncodableLine,
}





#[derive(Debug)]
pub struct PositionedEncodableBlock {
    pub addr: usize,
    pub name: SectionName,
    pub instructions: Vec<EncodableLine>
}

#[derive(Debug)]
pub struct PositionedEncodedBlock {
    pub addr: usize,
    pub name: SectionName,
    pub instructions: Vec<EncodedData>
}
