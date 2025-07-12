use crate::syntax::intel::{self};
use crate::machine;
use crate::cpu;

pub trait Executor<'a> {
    type Instruction;

    fn execute(&'a self, cpu: &mut impl cpu::CPU, inst: &'a Self::Instruction) -> ();
}

pub struct StatementExecutor;

impl<'a> Executor<'a> for StatementExecutor {
    type Instruction = intel::Statement<'a>;
    fn execute(&'a self, cpu: &mut impl cpu::CPU, inst: &'a Self::Instruction) -> () {
        match inst {
            intel::Statement::Label(s) => {

            },
            intel::Statement::Directive(s) => {

            },
            intel::Statement::PseudoInstruction{opcode, args} => {
                match opcode {
                    intel::Pseudo::RET => {
                        println!("Ret!");
                    },
                    intel::Pseudo::LI => {
                        println!("LI!");
                    },
                }
            },
            intel::Statement::Instruction{opcode, args} => {
                match opcode.get_format() {
                    crate::spec::Instruction::R { funct7, rs2, rs1, funct3, rd, opcode } => println!("Rtype"),
                    crate::spec::Instruction::I { imm, rs1, funct3, rd, opcode } => println!("Itype"),
                    crate::spec::Instruction::S { imm, rs2, rs1, funct3, opcode } => todo!(),
                    crate::spec::Instruction::B { imm, rs2, rs1, funct3, opcode } => todo!(),
                    crate::spec::Instruction::U { imm, rd, opcode } => todo!(),
                    crate::spec::Instruction::J { imm, rd, opcode } => todo!(),
                }
            },
        }
    }
}
