// Low Assembly Types are types which handle:
// * symbol resolution
// * absolute/relative address assignment
// * low level details (such as binary encoding/representation)

use crate::{
    lang::{
        directive::Directive,
        ext::Extension,
        highassembly::SectionName,
    }, streamreader::Position,
};

use super::ext::instruction_to_binary;




#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DataEndianness {
    Le,
    Be,
}

impl DataEndianness {
    pub fn build_words_from_bytes(bytes: &[u8], target: DataEndianness) -> Vec<u32> {
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

    pub fn build_word_from_bytes(bytes: [u8; 4], target: DataEndianness) -> u32 {
        match target {
            DataEndianness::Le => u32::from_le_bytes(bytes),
            DataEndianness::Be => u32::from_be_bytes(bytes),
        }
    }

    pub fn break_word_into_bytes(word: u32, target: DataEndianness) -> [u8; 4] {
        match target {
            DataEndianness::Le => u32::to_le_bytes(word),
            DataEndianness::Be => u32::to_be_bytes(word),
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

    pub fn modify_bytes(bytes: [u8; 4], source: DataEndianness, target: DataEndianness) -> [u8; 4] {
        if source == target {
            bytes
        }
        else {
            let conv = [bytes[3], bytes[2], bytes[1], bytes[0]];
            conv
        }
    }
}




#[derive(Debug)]
pub struct EncodedData {
    pub file_pos: Position,
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
    pub file_pos: Position,
    pub key: EncodableKey,
    pub args: Vec<i32>,
}

impl EncodableLine {
    pub fn encode(self) -> EncodedData {
        match self.key {
            EncodableKey::Op(op) => {
                let data = vec![instruction_to_binary(&op, &self.args)];
                EncodedData {
                    file_pos: self.file_pos,
                    data,
                    alignment: 4,
                }
            },
            EncodableKey::Directive(d) => {
                let alignment = d.datatype().alignment();
                let data: Vec<u32> = {
                    let args: Vec<u32> = self.args
                        .into_iter()
                        .map(|arg| arg as u32)
                        .collect();
                    args
                };
                EncodedData {
                    file_pos: self.file_pos,
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
