pub trait Assembler<'a> {
    type Instruction;
    fn to_words(&self, instructions: &'a Vec<Self::Instruction>) -> Vec<u32> ;
}

/**/

use crate::spec::{Extension, SyntaxField, Arg};

pub fn to_u32(instructions: &Vec<(&Box<dyn Extension>, Vec<Arg>)>) -> Vec<u32> {
    let mut v = Vec::new();
    for inst in instructions {
        match inst.0.get_syntax() {
            crate::spec::Syntax::N0 => todo!(),
            crate::spec::Syntax::N1(field) => todo!(),
            crate::spec::Syntax::N2(field, field1) => {
                let fields = vec![field, field1];
                let (rs1, rs2, rd, imm) = get_args(fields, &inst.1);
                v.push(inst.0.get_instruction(rs1, rs2, rd, imm).get_bytes());
            },
            crate::spec::Syntax::N3(field, field1, field2) => {
                let fields = vec![field, field1, field2];
                let (rs1, rs2, rd, imm) = get_args(fields, &inst.1);
                v.push(inst.0.get_instruction(rs1, rs2, rd, imm).get_bytes());
            },
            crate::spec::Syntax::N4(field, field1, field2, field3) => todo!(),
        }
    }
    v
}

fn get_args(
    fields: Vec<SyntaxField>,
    args: &Vec<Arg>
) -> (i32, i32, i32, i32)
{
    let mut rs1: i32 = 0;
    let mut rs2: i32 = 0;
    let mut rd: i32 = 0;
    let mut imm: i32 = 0;
    for (field, arg) in fields.iter().zip(args.iter()) {
        match arg {
            Arg::NUMBER(v) => imm = *v,
            Arg::REG(reg) => {
                match field {
                    SyntaxField::RS1 => rs1 = *reg,
                    SyntaxField::RS2 => rs2 = *reg,
                    SyntaxField::RD => rd = *reg,
                    _ => eprintln!("Error")
                }
            },
        }
    }
    (rs1, rs2, rd, imm)
}
