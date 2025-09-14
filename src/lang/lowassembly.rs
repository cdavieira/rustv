use crate::{lang::{
    directive::Directive, ext::Extension, highassembly::SectionName
}, utils::print_words_hex};

use super::ext::{
    ArgName,
    ArgSyntax,
};

// Post symbol resolution + Post address assignment + Post block creation




#[derive(Debug)]
pub struct EncodedData {
    pub data: Vec<u32>,
    pub alignment: usize,
}




// TODO: use this somehow

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
                    args
                        .chunks(4)
                        .map(|chunk| {
                            let word_bytes: [u8; 4] = chunk
                                .try_into()
                                .expect("Error encoding data for directive");
                            u32::from_le_bytes(word_bytes)
                        })
                        .collect()
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





pub fn instruction_to_binary(inst: &Box<dyn Extension>, args: &Vec<i32>) -> u32 {
    let fields = match inst.get_calling_syntax() {
        ArgSyntax::N0 => vec![],
        ArgSyntax::N1(f0) => vec![f0],
        ArgSyntax::N2(f0, f1) => vec![f0, f1],
        ArgSyntax::N3(f0, f1, f2) => vec![f0, f1, f2],
        ArgSyntax::N4(f0, f1, f2, f3) => vec![f0, f1, f2, f3],
    };
    let (rs1, rs2, rd, imm) = get_args(fields, args);
    inst.get_instruction_format(rs1, rs2, rd, imm).encode()
}

fn get_args(
    fields: Vec<ArgName>,
    args: &Vec<i32>
) -> (u32, u32, u32, i32)
{
    let mut rs1: u32 = 0;
    let mut rs2: u32 = 0;
    let mut rd:  u32 = 0;
    let mut imm: i32 = 0;
    for (field, arg) in fields.iter().zip(args.iter()) {
        match field {
            ArgName::RS1 => rs1 = (*arg) as u32,
            ArgName::RS2 => rs2 = (*arg) as u32,
            ArgName::RD =>  rd  = (*arg) as u32,
            ArgName::IMM | ArgName::OFF => imm = *arg,
        }
    }
    (rs1, rs2, rd, imm)
}
