use crate::syntax::intel::{self, Token, Pseudo};
use crate::{spec::Register};
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
            intel::Statement::Instruction{opcode, args} => {
                println!("Instruction!");
                match opcode {
                    intel::Command::RET => {
                        println!("Ret!");
                    },
                    intel::Command::PSEUDO(pseudo) => {
                        println!("Pseudo!");
                        match pseudo {
                            Pseudo::LI => {
                                li(cpu, args[0], &crate::syntax::intel::Token::REG(crate::spec::Register::X0), args[2]);
                            },
                            _ => {

                            }
                        }
                    },
                    intel::Command::OP(op) => {
                        println!("OP + Args!");
                        match op {
                            intel::Opcode::RV32I(rv32i) => {
                                println!("RV32I!");
                                match rv32i {
                                    crate::spec::extensions::rv32i::Opcode::ADDI => {
                                        println!("ADDI!");
                                        addi(cpu, args[0], args[2], args[4])
                                    },
                                    what => {
                                        println!("Something else: {:?}", what);
                                    }
                                }
                            }
                        }
                    },
                }
            },
        }
    }
}

fn addi(cpu: &mut impl cpu::CPU, dst: &Token, arg1: &Token, arg2: &Token) -> () {
    if let (Token::REG(r), Token::REG(a1), Token::REG(a2)) = (dst, arg1, arg2) {
        let v1 = cpu.read(a1.id().into());
        let v2 = cpu.read(a2.id().into());
        cpu.write(r.id().into(), v1 + v2);
    }
}

fn li(cpu: &mut impl cpu::CPU, dst: &Token, src: &Token, imm: &Token) -> () {
    if let (Token::REG(d), Token::REG(s), Token::NUMBER(i)) = (dst, src, imm) {
        let v1 = cpu.read(s.id().into());
        let i: u32 = (*i).try_into().unwrap();
        cpu.write(d.id().into(), v1 + i);
    }
}
