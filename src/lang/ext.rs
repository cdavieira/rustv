use crate::utils::{
    get_bits_from_to,
    get_n_bits_from,
    get_single_bit_at,
    set_remaining_bits,
};



// Available Instruction Immediate Formats (as in the ISA)

/*
Functions to convert an immediate as read from the parser into the number to be stored in an
Instruction.

The Instruction is going to use this result as-is later to assemble a 32-bit instruction.
*/

pub trait Immediate {
    fn encode(imm: u32) -> Self ;
    fn decode(&self) -> u32 ;
}

#[derive(Debug, Copy, Clone)]
pub struct ImmediateI(u32);

#[derive(Debug, Copy, Clone)]
pub struct ImmediateS(u32, u32);

#[derive(Debug, Copy, Clone)]
pub struct ImmediateB(u32, u32);

#[derive(Debug, Copy, Clone)]
pub struct ImmediateU(u32);

#[derive(Debug, Copy, Clone)]
pub struct ImmediateJ(u32);

impl Immediate for ImmediateI {
    fn encode(imm: u32) -> Self {
        ImmediateI (
            get_n_bits_from(&imm, 0, 12)
        )
    }

    fn decode(&self) -> u32 {
        let sign_bit = get_single_bit_at(self.0, 11) as usize;
        set_remaining_bits(self.0, 11, sign_bit)
    }
}

impl Immediate for ImmediateS {
    fn encode(imm: u32) -> Self {
        let imm1 = get_n_bits_from(&imm, 5, 7);
        let imm2 = get_n_bits_from(&imm, 0, 5);
        ImmediateS(imm1, imm2)
    }

    fn decode(&self) -> u32 {
        let imm = (self.0 << 5) | self.1;
        let sign_bit = get_single_bit_at(imm, 11) as usize;
        set_remaining_bits(imm, 11, sign_bit)
    }
}

impl Immediate for ImmediateB {
    fn encode(imm: u32) -> Self {
        let bit11 = get_single_bit_at(imm, 11);
        let bit12 = get_single_bit_at(imm, 12);
        let bits_5_10 = get_bits_from_to(imm, 5, 10);
        let bits_1_4  = get_bits_from_to(imm, 1, 4);
        let imm1  = (bit12     << 6) | bits_5_10;
        let imm2  = (bits_1_4  << 1) | bit11;
        ImmediateB(imm1, imm2)
    }

    fn decode(&self) -> u32 {
        let bit0     = get_single_bit_at(self.1, 0);
        let bit_1_4  = get_bits_from_to(self.1, 1, 4);
        let bit_5_10 = get_bits_from_to(self.0, 0, 5);
        let bit11    = get_single_bit_at(self.0, 6);
        let imm = ((bit_5_10 << 5) | (bit_1_4 << 1) | bit0) << 1;
        set_remaining_bits(imm, 12, bit11 as usize)
    }
}

impl Immediate for ImmediateU {
    fn encode(imm: u32) -> Self {
        let imm = get_n_bits_from(&imm, 0, 20);
        ImmediateU(imm)
    }

    fn decode(&self) -> u32  {
        self.0 << 12
    }
}

impl Immediate for ImmediateJ {
    fn encode(imm: u32) -> Self {
        let p1 = get_bits_from_to(imm, 12, 19);
        let p2 = get_single_bit_at(imm, 11);
        let p3 = get_bits_from_to(imm, 1, 10);
        let p4 = get_single_bit_at(imm, 20);
        let imm = (p4 << 19) | (p3 << 9) | (p2 << 8) | p1;
        ImmediateJ(imm)
    }

    fn decode(&self) -> u32 {
        let bit_1_10  = get_bits_from_to(self.0, 9, 18);
        let bit11     = get_single_bit_at(self.0, 8);
        let bit_12_19 = get_bits_from_to(self.0, 0, 7);
        let bit20     = get_single_bit_at(self.0, 19);
        let imm = ((bit_12_19 << 11) | (bit11 << 10) | bit_1_10) << 1;
        set_remaining_bits(imm, 20, bit20 as usize)
    }
}





// Available Instruction Binary Formats (as in the ISA)

#[derive(Debug, Copy, Clone)]
pub enum InstructionFormat {
    R{funct7: u32, rs2: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32},
    I{imm: ImmediateI, rs1: u32, funct3: u32, rd: u32, opcode: u32},
    S{imm: ImmediateS, rs2: u32, rs1: u32, funct3: u32, opcode: u32},
    B{imm: ImmediateB, rs2: u32, rs1: u32, funct3: u32, opcode: u32},
    U{imm: ImmediateU, rd: u32, opcode: u32},
    J{imm: ImmediateJ, rd: u32, opcode: u32},
}

impl InstructionFormat {
    pub fn decode(word: u32) -> Option<Self> {
        let opcode = get_n_bits_from(&word, 0, 7);
        match opcode {
            0b0110011 => { //R
                let rd      = get_n_bits_from(&word, 7, 5);
                let funct3  = get_n_bits_from(&word, 12, 3);
                let rs1     = get_n_bits_from(&word, 15, 5);
                let rs2     = get_n_bits_from(&word, 20, 5);
                let funct7  = get_n_bits_from(&word, 25, 7);
                Some(InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode })
            },
            0b0110111 | 0b0010111 => { //U
                let rd   = get_n_bits_from(&word, 7, 5);
                let imm  = get_n_bits_from(&word, 12, 20);
                Some(InstructionFormat::U { imm: ImmediateU(imm), rd, opcode })
            },
            0b1101111 => { //J
                let rd   = get_n_bits_from(&word, 7, 5);
                let imm  = get_n_bits_from(&word, 12, 20);
                Some(InstructionFormat::J { imm: ImmediateJ(imm), rd, opcode })
            },
            0b1100111 | 0b1110011 | 0b0010011 | 0b0000011 => { //I
                let rd      = get_n_bits_from(&word, 7, 5);
                let funct3  = get_n_bits_from(&word, 12, 3);
                let rs1     = get_n_bits_from(&word, 15, 5);
                let imm     = get_n_bits_from(&word, 20, 12);
                Some(InstructionFormat::I { imm: ImmediateI(imm), rs1, funct3, rd, opcode })
            },
            0b0100011  => { //S
                let imm2    = get_n_bits_from(&word, 7, 5);
                let funct3  = get_n_bits_from(&word, 12, 3);
                let rs1     = get_n_bits_from(&word, 15, 5);
                let rs2     = get_n_bits_from(&word, 20, 5);
                let imm1    = get_n_bits_from(&word, 25, 5);
                Some(InstructionFormat::S { imm: ImmediateS(imm1, imm2), rs2, rs1, funct3, opcode })
            },
            0b1100011 => { //B
                let imm2    = get_n_bits_from(&word, 7, 5);
                let funct3  = get_n_bits_from(&word, 12, 3);
                let rs1     = get_n_bits_from(&word, 15, 5);
                let rs2     = get_n_bits_from(&word, 20, 5);
                let imm1    = get_n_bits_from(&word, 25, 7);
                Some(InstructionFormat::B { imm: ImmediateB(imm1, imm2), rs2, rs1, funct3, opcode })
            }
            _ => {
                None
            }
        }
    }

    pub fn encode(&self) -> u32 {
        match self {
            InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode } => {
                let opcode = get_n_bits_from(opcode, 0, 7);
                let rd     = get_n_bits_from(rd, 0, 5);
                let rs1    = get_n_bits_from(rs1, 0, 5);
                let rs2    = get_n_bits_from(rs2, 0, 5);
                let funct3 = get_n_bits_from(funct3, 0, 3);
                let funct7 = get_n_bits_from(funct7, 0, 7);
                (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            InstructionFormat::I { imm, rs1, funct3, rd, opcode } => {
                let opcode = get_n_bits_from(opcode, 0, 7);
                let rd     = get_n_bits_from(rd, 0, 5);
                let rs1    = get_n_bits_from(rs1, 0, 5);
                let funct3 = get_n_bits_from(funct3, 0, 3);
                let imm    = get_n_bits_from(&imm.0, 0, 12);
                (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            InstructionFormat::S { imm, rs2, rs1, funct3, opcode } => {
                let opcode = get_n_bits_from(opcode, 0, 7);
                let rs1    = get_n_bits_from(rs1, 0, 5);
                let rs2    = get_n_bits_from(rs2, 0, 5);
                let funct3 = get_n_bits_from(funct3, 0, 3);
                let imm2   = get_n_bits_from(&imm.1, 0, 5);
                let imm1   = get_n_bits_from(&imm.0, 0, 7);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            InstructionFormat::B { imm, rs2, rs1, funct3, opcode } => {
                let opcode = get_n_bits_from(opcode, 0, 7);
                let rs1    = get_n_bits_from(rs1, 0, 5);
                let rs2    = get_n_bits_from(rs2, 0, 5);
                let funct3 = get_n_bits_from(funct3, 0, 3);
                let imm2   = get_n_bits_from(&imm.1, 0, 5);
                let imm1   = get_n_bits_from(&imm.0, 0, 7);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            InstructionFormat::U { imm, rd, opcode } => {
                let opcode = get_n_bits_from(opcode, 0, 7);
                let rd     = get_n_bits_from(rd, 0, 5);
                let imm    = get_n_bits_from(&imm.0, 0, 20);
                (imm << 12) | (rd << 7) | opcode
            },
            InstructionFormat::J { imm, rd, opcode } => {
                let opcode = get_n_bits_from(opcode, 0, 7);
                let rd     = get_n_bits_from(rd, 0, 5);
                let imm    = get_n_bits_from(&imm.0, 0, 20);
                (imm << 12) | (rd << 7) | opcode
            },
        }
    }

    pub fn r(funct7: u32, rs2: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> Self {
        InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode }
    }

    pub fn i(imm: i32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> Self {
        InstructionFormat::I { imm: ImmediateI::encode(imm as u32), rs1, funct3, rd, opcode }
    }

    pub fn s(imm: i32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> Self {
        InstructionFormat::S { imm: ImmediateS::encode(imm as u32), rs2, rs1, funct3, opcode }
    }

    pub fn b(imm: i32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> Self {
        InstructionFormat::B { imm: ImmediateB::encode(imm as u32), rs2, rs1, funct3, opcode }
    }

    pub fn u(imm: i32, rd: u32, opcode: u32) -> Self {
        InstructionFormat::U { imm: ImmediateU::encode(imm as u32), rd, opcode }
    }

    pub fn j(imm: i32, rd: u32, opcode: u32) -> Self {
        InstructionFormat::J { imm: ImmediateJ::encode(imm as u32), rd, opcode }
    }
}




// Instruction Assembly Description

pub enum ArgName {
    RS1,
    RS2,
    RD,
    IMM,
    OFF,
}

pub enum ArgSyntax {
    N0,
    N1(ArgName),
    N2(ArgName, ArgName),
    N3(ArgName, ArgName, ArgName),
    N4(ArgName, ArgName, ArgName, ArgName),
}




// Extensions

/** 
An extension was thought as a set of new instructions which can extend the functionalities
offered by the assembly language to access and interact with the hardware.

Each new instruction of the extension must have a format (as described in the riscv
specification), which can be found in the enum 'InstructionFormat'

In order for new extensions to be supported, it will be needed to:
    * create an entity for that extension (an enum/struct)
    * implement this trait
    * optionally create and link other entities to the recently created extension (or more)

This trait allows implementors to map their internal state (identifiers/keywords of new
instructions) to a instruction format

The implementer of this trait (an extension such RV32I, RV32E, RV64I, ...) will usually be an
enum whose variants are then linked to some specific instruction format.
*/
pub trait Extension: std::fmt::Debug {
    fn get_instruction_format(&self, rs1: u32, rs2: u32, rd: u32, imm: i32) -> InstructionFormat ;
    fn get_calling_syntax(&self) -> ArgSyntax ;
}





// Extension implementers

/** Implementing the extension RV32I (The Basic Instruction Set)

As an example of an extension yet to be added, it is required for the implementer to:
    * create an entity for this extension (the enum 'RV32I')
    * implement the trait 'Extension' for that entity

Again, the 'Extension' trait allows the implementer to link its internal state (the
identifiers/keywords for each instruction) to a specific instruction format

OBS: According to 'The RISC-V Instruction Set Manual - Volume 1 (Unpriviledged Architecture) -
Version 20250508', the RV32I includes 40 instructions, out of which 2 are usually left'd out (FENCE
and ECALL/EBREAK)
*/
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum RV32I {
    LUI, AUIPC, ADDI, ANDI, ORI, XORI, ADD, SUB, AND, OR, XOR, SLL, SRL, SRA, FENCE, SLTI, SLTIU,
    SLLI, SRLI, SRAI, SLT, SLTU, LW, LH, LHU, LB, LBU, SW, SH, SB, JAL, JALR, BEQ, BNE, BLT, BLTU,
    BGE, BGEU, ECALL
}

// TODO: check if 'imm' is in the valid interval according to the instruction type (I, for example)
impl Extension for RV32I {
    fn get_instruction_format(&self, rs1: u32, rs2: u32, rd: u32, imm: i32) -> InstructionFormat  {
        match self {
            RV32I::ADD   => InstructionFormat::r(0b0000000, rs2, rs1, 0b000, rd, 0b0110011),
            RV32I::SUB   => InstructionFormat::r(0b0100000, rs2, rs1, 0b000, rd, 0b0110011),
            RV32I::AND   => InstructionFormat::r(0b0000000, rs2, rs1, 0b111, rd, 0b0110011),
            RV32I::OR    => InstructionFormat::r(0b0000000, rs2, rs1, 0b110, rd, 0b0110011),
            RV32I::XOR   => InstructionFormat::r(0b0000000, rs2, rs1, 0b100, rd, 0b0110011),
            RV32I::SLL   => InstructionFormat::r(0b0000000, rs2, rs1, 0b001, rd, 0b0110011),
            RV32I::SRL   => InstructionFormat::r(0b0000000, rs2, rs1, 0b101, rd, 0b0110011),
            RV32I::SRA   => InstructionFormat::r(0b0100000, rs2, rs1, 0b101, rd, 0b0110011),
            RV32I::SLT   => InstructionFormat::r(0b0000000, rs2, rs1, 0b010, rd, 0b0110011),
            RV32I::SLTU  => InstructionFormat::r(0b0000000, rs2, rs1, 0b011, rd, 0b0110011),
            RV32I::LUI   => InstructionFormat::u(imm, rd, 0b0110111),
            RV32I::AUIPC => InstructionFormat::u(imm, rd, 0b0010111),
            RV32I::JAL   => InstructionFormat::j(imm, rd, 0b1101111),
            RV32I::JALR  => InstructionFormat::i(imm, rs1, 0b000, rd, 0b1100111),
            RV32I::ECALL => InstructionFormat::i(imm, rs1, 0b000, rd, 0b1110011),
            RV32I::ADDI  => InstructionFormat::i(imm, rs1, 0b000, rd, 0b0010011),
            RV32I::ANDI  => InstructionFormat::i(imm, rs1, 0b111, rd, 0b0010011),
            RV32I::ORI   => InstructionFormat::i(imm, rs1, 0b110, rd, 0b0010011),
            RV32I::XORI  => InstructionFormat::i(imm, rs1, 0b100, rd, 0b0010011),
            RV32I::SLTI  => InstructionFormat::i(imm, rs1, 0b010, rd, 0b0010011),
            RV32I::SLTIU => InstructionFormat::i(imm, rs1, 0b011, rd, 0b0010011),
            RV32I::SLLI  => InstructionFormat::i(0b00_00000_11111 & imm, rs1, 0b001, rd, 0b0010011),
            RV32I::SRLI  => InstructionFormat::i(0b00_00000_11111 & imm, rs1, 0b101, rd, 0b0010011),
            RV32I::SRAI  => InstructionFormat::i(0b01_00000_11111 & imm, rs1, 0b101, rd, 0b0010011),
            RV32I::LW    => InstructionFormat::i(imm, rs1, 0b010, rd,  0b0000011),
            RV32I::LH    => InstructionFormat::i(imm, rs1, 0b001, rd,  0b0000011),
            RV32I::LB    => InstructionFormat::i(imm, rs1, 0b000, rd,  0b0000011),
            RV32I::LHU   => InstructionFormat::i(imm, rs1, 0b101, rd,  0b0000011),
            RV32I::LBU   => InstructionFormat::i(imm, rs1, 0b100, rd,  0b0000011),
            RV32I::SW    => InstructionFormat::s(imm, rs2, rs1, 0b010, 0b0100011),
            RV32I::SH    => InstructionFormat::s(imm, rs2, rs1, 0b001, 0b0100011),
            RV32I::SB    => InstructionFormat::s(imm, rs2, rs1, 0b000, 0b0100011),
            RV32I::BEQ   => InstructionFormat::b(imm, rs2, rs1, 0b000, 0b1100011),
            RV32I::BNE   => InstructionFormat::b(imm, rs2, rs1, 0b001, 0b1100011),
            RV32I::BLT   => InstructionFormat::b(imm, rs2, rs1, 0b100, 0b1100011),
            RV32I::BLTU  => InstructionFormat::b(imm, rs2, rs1, 0b110, 0b1100011),
            RV32I::BGE   => InstructionFormat::b(imm, rs2, rs1, 0b101, 0b1100011),
            RV32I::BGEU  => InstructionFormat::b(imm, rs2, rs1, 0b111, 0b1100011),
            RV32I::FENCE => todo!(),
        }
    }

    fn get_calling_syntax(&self) -> ArgSyntax {
        match self {
            RV32I::ADD   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::SUB   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::AND   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::OR    => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::XOR   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::SLL   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::SRL   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::SRA   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::SLT   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::SLTU  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::RS2),
            RV32I::LUI   => ArgSyntax::N2(ArgName::RD, ArgName::IMM),
            RV32I::AUIPC => ArgSyntax::N2(ArgName::RD, ArgName::IMM),
            RV32I::JAL   => ArgSyntax::N2(ArgName::RD, ArgName::OFF),
            RV32I::JALR  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::OFF),
            RV32I::ECALL => ArgSyntax::N0,
            RV32I::ADDI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::ANDI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::ORI   => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::XORI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::SLTI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::SLTIU => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::SLLI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::SRLI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::SRAI  => ArgSyntax::N3(ArgName::RD, ArgName::RS1, ArgName::IMM),
            RV32I::LW    => ArgSyntax::N3(ArgName::RD, ArgName::OFF, ArgName::RS1),
            RV32I::LH    => ArgSyntax::N3(ArgName::RD, ArgName::OFF, ArgName::RS1),
            RV32I::LB    => ArgSyntax::N3(ArgName::RD, ArgName::OFF, ArgName::RS1),
            RV32I::LHU   => ArgSyntax::N3(ArgName::RD, ArgName::OFF, ArgName::RS1),
            RV32I::LBU   => ArgSyntax::N3(ArgName::RD, ArgName::OFF, ArgName::RS1),
            RV32I::SW    => ArgSyntax::N3(ArgName::RS2, ArgName::OFF, ArgName::RS1),
            RV32I::SH    => ArgSyntax::N3(ArgName::RS2, ArgName::OFF, ArgName::RS1),
            RV32I::SB    => ArgSyntax::N3(ArgName::RS2, ArgName::OFF, ArgName::RS1),
            RV32I::BEQ   => ArgSyntax::N3(ArgName::RS1, ArgName::RS2, ArgName::OFF),
            RV32I::BNE   => ArgSyntax::N3(ArgName::RS1, ArgName::RS2, ArgName::OFF),
            RV32I::BLT   => ArgSyntax::N3(ArgName::RS1, ArgName::RS2, ArgName::OFF),
            RV32I::BLTU  => ArgSyntax::N3(ArgName::RS1, ArgName::RS2, ArgName::OFF),
            RV32I::BGE   => ArgSyntax::N3(ArgName::RS1, ArgName::RS2, ArgName::OFF),
            RV32I::BGEU  => ArgSyntax::N3(ArgName::RS1, ArgName::RS2, ArgName::OFF),
            RV32I::FENCE => todo!(),
        }
    }
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
    let iformat = inst.get_instruction_format(rs1, rs2, rd, imm);
    iformat.encode()
}

fn get_args(
    fields: Vec<ArgName>,
    args: &Vec<i32>
) -> (u32, u32, u32, i32)
{
    if fields.len() != args.len() {
        panic!("Insuficient number of arguments");
    }

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
