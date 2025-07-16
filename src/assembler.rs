pub trait Assembler {
    type Instruction;
    fn to_words(&self, instructions: Vec<Self::Instruction>) -> Vec<u32> ;
}

/**/

use crate::spec::{Extension, ArgKey, ArgValue};

pub fn to_u32(instructions: Vec<(Box<dyn Extension>, Vec<ArgValue>)>) -> Vec<u32> {
    let mut v = Vec::new();
    for inst in instructions {
        match inst.0.get_syntax() {
            crate::spec::ArgSyntax::N0 => todo!(),
            crate::spec::ArgSyntax::N1(field) => todo!(),
            crate::spec::ArgSyntax::N2(field, field1) => {
                let fields = vec![field, field1];
                let (rs1, rs2, rd, imm) = get_args(fields, &inst.1);
                v.push(inst.0.get_instruction(rs1, rs2, rd, imm).get_bytes());
            },
            crate::spec::ArgSyntax::N3(field, field1, field2) => {
                let fields = vec![field, field1, field2];
                let (rs1, rs2, rd, imm) = get_args(fields, &inst.1);
                v.push(inst.0.get_instruction(rs1, rs2, rd, imm).get_bytes());
            },
            crate::spec::ArgSyntax::N4(field, field1, field2, field3) => todo!(),
        }
    }
    v
}

fn get_args(
    fields: Vec<ArgKey>,
    args: &Vec<ArgValue>
) -> (i32, i32, i32, i32)
{
    let mut rs1: i32 = 0;
    let mut rs2: i32 = 0;
    let mut rd: i32 = 0;
    let mut imm: i32 = 0;
    for (field, arg) in fields.iter().zip(args.iter()) {
        match arg {
            ArgValue::NUMBER(v) => imm = *v,
            ArgValue::REG(reg) => {
                match field {
                    ArgKey::RS1 => rs1 = *reg,
                    ArgKey::RS2 => rs2 = *reg,
                    ArgKey::RD => rd = *reg,
                    _ => eprintln!("Error")
                }
            },
        }
    }
    (rs1, rs2, rd, imm)
}
