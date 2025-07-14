pub trait Assembler<'a> {
    type Output;
    fn to_words(&self, instructions: Vec<&'a Self::Output>) -> Vec<u32> ;
}

/**/
pub trait HandleArg<'a, 'b> {
    type Token;
    fn is_register(&self, token: &Self::Token) -> bool;
    fn is_offset(&self, token: &Self::Token) -> bool;
    fn is_immediate(&self, token: &Self::Token) -> bool;

    fn read_register(&self, args: &mut impl Iterator<Item = &'a &'b Self::Token>) -> Option<&'a &'b Self::Token> {
        let mut arg_opt = args.next();
        while let Some(arg) = &arg_opt {
            if self.is_register(arg) {
                break;
            }
            arg_opt = args.next();
        }
        arg_opt
    }
    fn read_offset(&self, args: &mut impl Iterator<Item = &'a &'b Self::Token>) -> Option<&'a &'b Self::Token> {
        let mut arg_opt = args.next();
        while let Some(arg) = &arg_opt {
            if self.is_offset(arg) {
                break;
            }
            arg_opt = args.next();
        }
        arg_opt
    }
    fn read_immediate(&self, args: &mut impl Iterator<Item = &'a &'b Self::Token>) -> Option<&'a &'b Self::Token> {
        let mut arg_opt = args.next();
        while let Some(arg) = &arg_opt {
            if self.is_immediate(arg) {
                break;
            }
            arg_opt = args.next();
        }
        arg_opt
    }

    fn get_number(&self, token: &Self::Token) -> i32;
}

use crate::syntax::intel::Statement;
use crate::syntax::intel::Token;
use crate::spec::Field;
pub fn to_words<'a, 'b: 'a>(handler: &impl HandleArg<'a, 'b, Token = Token>, stats: Vec<&'a Statement<'b>>) -> Vec<u32> {
    let mut v = Vec::new();
    let mut it = stats.iter();
    let mut opt = it.next();
    while let Some(s) = opt {
        match s {
            Statement::PseudoInstruction { opcode, args } => todo!(),
            Statement::Instruction { opcode, args } => {
                match opcode.get_syntax() {
                    crate::spec::Syntax::N0 => todo!(),
                    crate::spec::Syntax::N1(field) => todo!(),
                    crate::spec::Syntax::N2(field, field1) => {
                        let fields = vec![field, field1];
                        let (rs1, rs2, rd, imm) = get_args(handler, fields, args);
                        v.push(opcode.get_instruction(rs1, rs2, rd, imm).get_bytes());
                    },
                    crate::spec::Syntax::N3(field, field1, field2) => {
                        let fields = vec![field, field1, field2];
                        let (rs1, rs2, rd, imm) = get_args(handler, fields, args);
                        v.push(opcode.get_instruction(rs1, rs2, rd, imm).get_bytes());
                    },
                    crate::spec::Syntax::N4(field, field1, field2, field3) => todo!(),
                }
            },
            Statement::Directive(_) => todo!(),
            Statement::Label(_) => todo!(),
        }
        opt = it.next();
    }
    v
}

fn get_args<'a, 'b: 'a>(
    handler: &impl HandleArg<'a, 'b, Token = Token>,
    fields: Vec<Field>,
    args: &'a Vec<&'b Token>
) -> (i32, i32, i32, i32)
{
    let mut args_it = args.iter();
    let mut rs1: i32 = 0;
    let mut rs2: i32 = 0;
    let mut rd: i32 = 0;
    let mut imm: i32 = 0;
    for f in fields {
        match f {
            Field::RS1 => {
                if let Some(reg) = handler.read_register(&mut args_it) {
                    rs1 = handler.get_number(&reg);
                }
            },
            Field::RS2 => {
                if let Some(reg) = handler.read_register(&mut args_it) {
                    rs2 = handler.get_number(&reg);
                }
            },
            Field::RD => {
                if let Some(reg) = handler.read_register(&mut args_it) {
                    rd = handler.get_number(&reg);
                }
            },
            Field::IMM => {
                if let Some(n) = handler.read_immediate(&mut args_it) {
                    imm = handler.get_number(&n);
                }
            },
            Field::OFF => {
                if let Some(o) = handler.read_offset(&mut args_it) {
                    imm = handler.get_number(&o);
                }
            },
        }
    }
    (rs1, rs2, rd, imm)
}
