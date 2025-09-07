use crate::lang::{
    ext::Extension,
    directive::Directive,
    highassembly::SectionName,
};

use super::ext::{
    ArgName,
    ArgSyntax,
};

// Post symbol resolution + Post address assignment + Post block creation





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
    pub fn encode(self) -> Vec<u32> {
        match self.key {
            EncodableKey::Op(op) => {
                vec![instruction_to_binary(&op, &self.args)]
            },
            EncodableKey::Directive(_) => {
                self.args
                        .iter()
                        .map(|x| *x as u32)
                        .collect()
            },
        }
    }
}





pub struct PositionedEncodableLine {
    pub addr: usize,
    pub line: EncodableLine,
}





#[derive(Debug)]
pub struct EncodableBlock {
    pub addr: usize,
    pub name: SectionName,
    pub instructions: Vec<EncodableLine>
}

#[derive(Debug)]
pub struct EncodedBlock {
    pub addr: usize,
    pub name: SectionName,
    pub instructions: Vec<u32>
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
