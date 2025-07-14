pub trait Assembler<'a> {
    type Output;
    fn to_words(&self, instructions: &'a Vec<Self::Output>) -> Vec<u32> ;
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




// pub trait TranslatePseudo<'a> {
// // pub trait TranslatePseudo {
//     type Token;
//     fn translate_pseudo(&self, stat: &Vec<&Self::Token>, pseudo_stat: &'a mut Vec<Self::Token>) -> Option<Vec<Vec<&'a Self::Token>>>;
//     // fn translate_pseudo(&self, stat: &Vec<&Self::Token>) -> Option<Vec<Vec<&Self::Token>>>;
// }
//
// // pub fn process_pseudos<'a, T>(translator: &'a impl TranslatePseudo<Token = T>, stats: Vec<Vec<&'a T>>) -> Vec<Vec<&'a T>> {
// pub fn process_pseudos<'a, T>(
//     translator: &'a impl TranslatePseudo<'a, Token = T>,
//     stats: &'a Vec<Vec<&'a T>>,
//     pseudo_stats: &'a mut Vec<T>,
//     new_stats: &mut Vec<&'a Vec<&'a T>>,
// ) -> ()
// {
//     new_stats.clear();
//     for stat in stats  {
//         if let Some(translated) = translator.translate_pseudo(stat, pseudo_stats) {
//             new_stats.extend(&translated);
//         }
//         else {
//             new_stats.push(stat);
//         }
//     }
// }

