use crate::lang::ext::{
    Extension,
    RV32I,
};

use crate::lang::highassembly::{
    ArgValue,
    OpcodeLine,
    Register,
};





pub trait Pseudo: std::fmt::Debug {
    fn translate(&self, args: Vec<ArgValue>) -> Vec<OpcodeLine> ;
}






// Pseudo Instructions implementation

#[derive(Debug, Copy, Clone)]
pub enum PseudoInstruction {
    LI,
    RET,
    MV,
    LA,
}

impl Pseudo for PseudoInstruction {
    fn translate(&self, args: Vec<ArgValue>) -> Vec<OpcodeLine>  {
        match self {
            // li x7 2, becomes:
            // addi x7 x0 2
            //
            // li x7 6144, becomes:
            // lui x7 1
            // addi x7 x7 2048
            PseudoInstruction::LI => {
                let arg1 = args.get(0).unwrap();
                let arg2 = args.get(1).unwrap();
                match (arg1, arg2) {
                    (ArgValue::Register(_), ArgValue::Number(n)) => {
                        if fits_in_12bit_immediate(*n) {
                            //'li' gets simplified to a 'addi' op
                            let addi_line = OpcodeLine {
                                keyword: Box::new(RV32I::ADDI),
                                args: vec![
                                    arg1.clone(),
                                    ArgValue::Register(Register::X0),
                                    ArgValue::Number(lower_12_bits(*n))
                                ],
                            };
                            return vec![addi_line];
                        }
                        else {
                            //Otherwise we have to:
                            //1. load the upper 20 bits of the immediate using 'lui'
                            //2. add the upper 20 bits with 'addi'
                            let lui_line = OpcodeLine {
                                keyword: Box::new(RV32I::LUI),
                                args: vec![
                                    arg1.clone(),
                                    ArgValue::Number(upper_20_bits(*n))
                                ],
                            };
                            let addi_line = OpcodeLine {
                                keyword: Box::new(RV32I::ADDI),
                                args: vec![
                                    arg1.clone(),
                                    arg1.clone(),
                                    ArgValue::Number(lower_12_bits(*n))
                                ],
                            };
                            return vec![lui_line, addi_line];
                        }
                    }
                    _ => {

                    }
                }
            },

            //ret, becomes:
            //jalr x0 x1 0
            PseudoInstruction::RET => {
                let jalr_line = OpcodeLine {
                    keyword: Box::new(RV32I::JALR),
                    args: vec![
                        ArgValue::Register(Register::X0),
                        ArgValue::Register(Register::X1),
                        ArgValue::Number(0),
                    ],
                };
                return vec![jalr_line];
            },

            //mv rd, rs1, becomes:
            //addi rd, rs1, 0
            PseudoInstruction::MV => {
                let arg1: ArgValue = args.get(0).unwrap().clone();
                let arg2: ArgValue = args.get(1).unwrap().clone();
                let addi_line = OpcodeLine {
                    keyword: Box::new(RV32I::ADDI),
                    args: vec![
                        arg1,
                        arg2,
                        ArgValue::Number(0),
                    ],
                };
                return vec![addi_line];
            },

            // la x5, my_label, becomes:
            // auipc x5, %hi(my_label) //20 bits
            // addi x5, x5, %lo(my_label) //12 bits
            //
            // or if %hi(my_label) == 0:
            // addi x5, x0, %lo(my_label) //12 bits
            PseudoInstruction::LA => {
                let arg1 = args.get(0).unwrap();
                let arg2 = args.get(1).unwrap();

                // TODO: remove the use of a number as the second argument
                match (arg1, arg2) {
                    (ArgValue::Register(_), ArgValue::Number(n)) => {
                        if fits_in_12bit_immediate(*n) {
                            let addi_line = OpcodeLine {
                                keyword: Box::new(RV32I::ADDI),
                                args: vec![
                                    arg1.clone(),
                                    ArgValue::Register(Register::X0),
                                    ArgValue::Number(lower_12_bits(*n))
                                ],
                            };
                            return vec![addi_line];
                        }
                        else {
                            let auipc_line = OpcodeLine {
                                keyword: Box::new(RV32I::AUIPC),
                                args: vec![
                                    arg1.clone(),
                                    ArgValue::Number(upper_20_bits(*n))
                                ],
                            };
                            let addi_line = OpcodeLine {
                                keyword: Box::new(RV32I::ADDI),
                                args: vec![
                                    arg1.clone(),
                                    arg1.clone(),
                                    ArgValue::Number(lower_12_bits(*n))
                                ],
                            };
                            return vec![auipc_line, addi_line];
                        }
                    },
                    (ArgValue::Register(_), ArgValue::Use(s)) => {
                        //We can't know if HI is 0 or not, therefore we can't optimize
                        let upper20bits = ArgValue::UseHi(s.to_string());
                        let lower12bits = ArgValue::UseLo(s.to_string());
                        let auipc_line = OpcodeLine {
                            keyword: Box::new(RV32I::AUIPC),
                            args: vec![
                                arg1.clone(),
                                upper20bits,
                            ],
                        };
                        let addi_line = OpcodeLine {
                            keyword: Box::new(RV32I::ADDI),
                            args: vec![
                                arg1.clone(),
                                arg1.clone(),
                                lower12bits,
                            ],
                        };
                        return vec![auipc_line, addi_line];
                    },
                    _ => {

                    }
                }
            }
        }

        Vec::new()
    }
}

fn fits_in_12bit_immediate(n: i32) -> bool {
    (n >= -2048) && (n <= 2047)
}

fn upper_20_bits(n: i32) -> i32 {
    (n >> 12) & 0b11111_11111_11111_11111
}

fn lower_12_bits(n: i32) -> i32 {
    n & 0b1111_1111_1111
}
