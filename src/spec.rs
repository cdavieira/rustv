// Registers

#[derive(Debug, Copy, Clone)]
pub enum Register {
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15,
    X16, X17, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, X31,
    PC,
}

impl Register {
    pub fn id(&self) -> u8 {
        match self {
            Register::X0  => 0,
            Register::X1  => 1,
            Register::X2  => 2,
            Register::X3  => 3,
            Register::X4  => 4,
            Register::X5  => 5,
            Register::X6  => 6,
            Register::X7  => 7,
            Register::X8  => 8,
            Register::X9  => 9,
            Register::X10 => 10,
            Register::X11 => 11,
            Register::X12 => 12,
            Register::X13 => 13,
            Register::X14 => 14,
            Register::X15 => 15,
            Register::X16 => 16,
            Register::X17 => 17,
            Register::X18 => 18,
            Register::X19 => 19,
            Register::X20 => 20,
            Register::X21 => 21,
            Register::X22 => 22,
            Register::X23 => 23,
            Register::X24 => 24,
            Register::X25 => 25,
            Register::X26 => 26,
            Register::X27 => 27,
            Register::X28 => 28,
            Register::X29 => 29,
            Register::X30 => 30,
            Register::X31 => 31,
            Register::PC  => todo!(),
        }
    }
}



// Available Instruction Binary Formats (as in the ISA)

use crate::utils::rsh_mask_bits;

#[derive(Debug, Copy, Clone)]
pub enum InstructionFormat {
    R{funct7: u32, rs2: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32},
    I{imm: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32},
    S{imm1: u32, rs2: u32, rs1: u32, funct3: u32, imm2: u32, opcode: u32},
    B{imm1: u32, rs2: u32, rs1: u32, funct3: u32, imm2: u32, opcode: u32},
    U{imm: u32, rd: u32, opcode: u32},
    J{imm: u32, rd: u32, opcode: u32},
}

impl InstructionFormat {
    pub fn decode(word: u32) -> Self {
        let opcode = rsh_mask_bits(&word, 0, 7);
        match opcode {
            0b0110011 => { //R
                let rd      = rsh_mask_bits(&word, 7, 5);
                let funct3  = rsh_mask_bits(&word, 12, 3);
                let rs1     = rsh_mask_bits(&word, 15, 5);
                let rs2     = rsh_mask_bits(&word, 20, 5);
                let funct7  = rsh_mask_bits(&word, 25, 7);
                InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode }
            },
            0b0110111 | 0b0010111 => { //U
                let rd   = rsh_mask_bits(&word, 7, 5);
                let imm  = rsh_mask_bits(&word, 12, 20);
                InstructionFormat::U { imm, rd, opcode }
            },
            0b1101111 => { //J
                let rd   = rsh_mask_bits(&word, 7, 5);
                let imm  = rsh_mask_bits(&word, 12, 20);
                InstructionFormat::J { imm, rd, opcode }
            },
            0b1100111 | 0b1110011 | 0b0010011 | 0b0000011 => { //I
                let rd      = rsh_mask_bits(&word, 7, 5);
                let funct3  = rsh_mask_bits(&word, 12, 3);
                let rs1     = rsh_mask_bits(&word, 15, 5);
                let imm     = rsh_mask_bits(&word, 20, 12);
                InstructionFormat::I { imm, rs1, funct3, rd, opcode }
            },
            0b0100011  => { //S
                let imm2    = rsh_mask_bits(&word, 7, 5);
                let funct3  = rsh_mask_bits(&word, 12, 3);
                let rs1     = rsh_mask_bits(&word, 15, 5);
                let rs2     = rsh_mask_bits(&word, 20, 5);
                let imm1    = rsh_mask_bits(&word, 25, 5);
                InstructionFormat::S { imm1, rs2, rs1, funct3, imm2, opcode }
            },
            0b1100011 => { //B
                let imm2    = rsh_mask_bits(&word, 7, 5);
                let funct3  = rsh_mask_bits(&word, 12, 3);
                let rs1     = rsh_mask_bits(&word, 15, 5);
                let rs2     = rsh_mask_bits(&word, 20, 5);
                let imm1    = rsh_mask_bits(&word, 25, 7);
                InstructionFormat::B { imm1, rs2, rs1, funct3, imm2, opcode }
            }
            _ => {
                panic!("InstructionFormat decode: unknown opcode: {}", opcode);
            }
        }
    }

    pub fn encode(&self) -> u32 {
        match self {
            InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode } => {
                let opcode = rsh_mask_bits(opcode, 0, 7);
                let rd     = rsh_mask_bits(rd, 0, 5);
                let rs1    = rsh_mask_bits(rs1, 0, 5);
                let rs2    = rsh_mask_bits(rs2, 0, 5);
                let funct3 = rsh_mask_bits(funct3, 0, 3);
                let funct7 = rsh_mask_bits(funct7, 0, 7);
                (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            InstructionFormat::I { imm, rs1, funct3, rd, opcode } => {
                let opcode = rsh_mask_bits(opcode, 0, 7);
                let rd     = rsh_mask_bits(rd, 0, 5);
                let rs1    = rsh_mask_bits(rs1, 0, 5);
                let funct3 = rsh_mask_bits(funct3, 0, 3);
                let imm    = rsh_mask_bits(imm, 0, 12);
                (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            InstructionFormat::S { imm1, rs2, rs1, funct3, imm2, opcode } => {
                let opcode = rsh_mask_bits(opcode, 0, 7);
                let rs1    = rsh_mask_bits(rs1, 0, 5);
                let rs2    = rsh_mask_bits(rs2, 0, 5);
                let funct3 = rsh_mask_bits(funct3, 0, 3);
                let imm2   = rsh_mask_bits(imm2, 0, 5);
                let imm1   = rsh_mask_bits(imm1, 0, 7);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            InstructionFormat::B { imm1, rs2, rs1, funct3, imm2, opcode } => {
                let opcode = rsh_mask_bits(opcode, 0, 7);
                let rs1    = rsh_mask_bits(rs1, 0, 5);
                let rs2    = rsh_mask_bits(rs2, 0, 5);
                let funct3 = rsh_mask_bits(funct3, 0, 3);
                let imm2   = rsh_mask_bits(imm2, 0, 5);
                let imm1   = rsh_mask_bits(imm1, 0, 7);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            InstructionFormat::U { imm, rd, opcode } => {
                let opcode = rsh_mask_bits(opcode, 0, 7);
                let rd     = rsh_mask_bits(rd, 0, 5);
                let imm    = rsh_mask_bits(imm, 0, 20);
                (imm << 12) | (rd << 7) | opcode
            },
            InstructionFormat::J { imm, rd, opcode } => {
                let opcode = rsh_mask_bits(opcode, 0, 7);
                let rd     = rsh_mask_bits(rd, 0, 5);
                let imm    = rsh_mask_bits(imm, 0, 20);
                (imm << 12) | (rd << 7) | opcode
            },
        }
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




// Assembly Instruction

#[derive(Debug)]
pub enum KeyValue {
    OP(Box<dyn Extension>),
    PSEUDO(Box<dyn Pseudo>),
    DIRECTIVE(Box<dyn Directive>),
    SECTION(AssemblySectionName),
    LABEL(String),
}

#[derive(Clone, Debug)]
pub enum ArgValue {
    BYTE(u8),
    NUMBER(i32),
    REGISTER(Register),
    OFFSET(usize, i32),
    LABEL(String),
    LITERAL(String),
    USE(String),
    USEHI(String),
    USELO(String),
}

#[derive(Debug)]
pub struct AssemblyInstruction {
    pub addr: usize,
    pub key: KeyValue,
    pub args: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssemblySectionName {
    TEXT,
    DATA,
    BSS,
    CUSTOM(String)
}

#[derive(Debug)]
pub struct AssemblySection {
    pub addr: usize,
    pub name: AssemblySectionName,
    pub instructions: Vec<AssemblyInstruction>
}

#[derive(Debug)]
pub struct AssemblyData {
    pub addr: usize,
    pub name: AssemblySectionName,
    pub data: Vec<u32>
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




// Pseudo Instructions

/**
A pseudoinstruction was thought to be a sequence of tokens which can be turned into a sequence of
Instructions.

In this regard, a instruction is an opcode, followed by one or more arguments
*/

pub trait Pseudo: std::fmt::Debug {
    fn translate(&self, args: Vec<ArgValue>) -> Vec<(Box<dyn Extension>, Vec<ArgValue>)> ;
}




// Directives

/**
A directive was thought to be a sequence of tokens which can be turned into a sequence of
raw bytes (4-byte aligned)

The length of the resulting vector is expected to be a multiple of 4 (to ensure 4bytes alignment)
*/

pub trait Directive: std::fmt::Debug {
    fn translate(&self, args: &Vec<ArgValue>) -> Vec<u8> ;
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

impl Extension for RV32I {
    fn get_instruction_format(&self, rs1: u32, rs2: u32, rd: u32, imm: i32) -> InstructionFormat  {
        match self {
            RV32I::ADD   => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b000, rd, opcode: 0b0110011 },
            RV32I::SUB   => InstructionFormat::R { funct7: 0b1000000, rs2, rs1, funct3: 0b000, rd, opcode: 0b0110011 },
            RV32I::AND   => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b111, rd, opcode: 0b0110011 },
            RV32I::OR    => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b110, rd, opcode: 0b0110011 },
            RV32I::XOR   => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b100, rd, opcode: 0b0110011 },
            RV32I::SLL   => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b001, rd, opcode: 0b0110011 },
            RV32I::SRL   => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b101, rd, opcode: 0b0110011 },
            RV32I::SRA   => InstructionFormat::R { funct7: 0b0100000, rs2, rs1, funct3: 0b101, rd, opcode: 0b0110011 },
            RV32I::SLT   => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b010, rd, opcode: 0b0110011 },
            RV32I::SLTU  => InstructionFormat::R { funct7: 0b0000000, rs2, rs1, funct3: 0b011, rd, opcode: 0b0110011 },
            RV32I::LUI   => InstructionFormat::U { imm: imm_to_u(imm), rd, opcode: 0b0110111 },
            RV32I::AUIPC => InstructionFormat::U { imm: imm_to_u(imm), rd, opcode: 0b0010111 },
            RV32I::JAL   => InstructionFormat::J { imm: imm_to_j(imm), rd, opcode: 0b1101111 },
            RV32I::JALR  => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b1100111 },
            RV32I::ECALL => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b1110011 },
            RV32I::ADDI  => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b0010011 },
            RV32I::ANDI  => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b111, rd, opcode: 0b0010011 },
            RV32I::ORI   => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b110, rd, opcode: 0b0010011 },
            RV32I::XORI  => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b100, rd, opcode: 0b0010011 },
            RV32I::SLTI  => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b010, rd, opcode: 0b0010011 },
            RV32I::SLTIU => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b011, rd, opcode: 0b0010011 },
            RV32I::SLLI  => InstructionFormat::I { imm: imm_to_i(0b00_00000_11111 & imm), rs1, funct3: 0b001, rd, opcode: 0b0010011 },
            RV32I::SRLI  => InstructionFormat::I { imm: imm_to_i(0b00_00000_11111 & imm), rs1, funct3: 0b101, rd, opcode: 0b0010011 },
            RV32I::SRAI  => InstructionFormat::I { imm: imm_to_i(0b01_00000_11111 & imm), rs1, funct3: 0b101, rd, opcode: 0b0010011 },
            RV32I::LW    => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b010, rd, opcode: 0b0000011 },
            RV32I::LH    => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b001, rd, opcode: 0b0000011 },
            RV32I::LB    => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b0000011 },
            RV32I::LHU   => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b101, rd, opcode: 0b0000011 },
            RV32I::LBU   => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b100, rd, opcode: 0b0000011 },
            RV32I::SW    => {
                let (imm1, imm2) = imm_to_s(imm);
                InstructionFormat::S { imm1, rs2, rs1, funct3: 0b010, imm2, opcode: 0b0100011 }
            },
            RV32I::SH    => {
                let (imm1, imm2) = imm_to_s(imm);
                InstructionFormat::S { imm1, rs2, rs1, funct3: 0b001, imm2, opcode: 0b0100011 }
            },
            RV32I::SB    => {
                let (imm1, imm2) = imm_to_s(imm);
                InstructionFormat::S { imm1, rs2, rs1, funct3: 0b000, imm2, opcode: 0b0100011 }
            },
            RV32I::BEQ   => {
                let (imm1, imm2) = imm_to_b(imm);
                InstructionFormat::B { imm1, rs2, rs1, funct3: 0b000, imm2, opcode: 0b1100011 }
            },
            RV32I::BNE   => {
                let (imm1, imm2) = imm_to_b(imm);
                InstructionFormat::B { imm1, rs2, rs1, funct3: 0b001, imm2, opcode: 0b1100011 }
            },
            RV32I::BLT   => {
                let (imm1, imm2) = imm_to_b(imm);
                InstructionFormat::B { imm1, rs2, rs1, funct3: 0b100, imm2, opcode: 0b1100011 }
            },
            RV32I::BLTU  => {
                let (imm1, imm2) = imm_to_b(imm);
                InstructionFormat::B { imm1, rs2, rs1, funct3: 0b110, imm2, opcode: 0b1100011 }
            },
            RV32I::BGE   => {
                let (imm1, imm2) = imm_to_b(imm);
                InstructionFormat::B { imm1, rs2, rs1, funct3: 0b101, imm2, opcode: 0b1100011 }
            },
            RV32I::BGEU  => {
                let (imm1, imm2) = imm_to_b(imm);
                InstructionFormat::B { imm1, rs2, rs1, funct3: 0b111, imm2, opcode: 0b1100011 }
            },
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

/*
Functions to convert an immediate as read from the parser into the number to be stored in an
Instruction.

The Instruction is going to use this result as-is later to assemble a 32-bit instruction.

Yet to be done:
    1. handle sign extension
    2. turn this into a struct/enum named 'InstructionImmediate'/'Immediate' ?
    3. use 'rsh_mask_bits' instead of raw bit manipulation?
*/

fn imm_to_i(imm: i32) -> u32 {
    (imm & 0b1111_1111_1111) as u32
}

fn imm_to_s(imm: i32) -> (u32, u32) {
    let imm1 = (imm & 0b1111_111_00000) >> 5;
    let imm2 = imm & 0b11111;
    (imm1 as u32, imm2 as u32)
}

fn imm_to_b(imm: i32) -> (u32, u32) {
    let bit12 = imm & 0b100_000_000_000;
    let bit13 = imm & 0b1_000_000_000_000;
    let imm1 = ((imm & 0b111111_00000) >> 5) | (bit13 >> 6);
    let imm2 = (imm & 0b11110) | (bit12 >> 11);
    (imm1 as u32, imm2 as u32)
}

fn imm_to_u(imm: i32) -> u32 {
    // (imm >> 12) & 0b11111_11111_11111_11111
    (imm & 0b11111_11111_11111_11111) as u32
}

fn imm_to_j(imm: i32) -> u32 {
    let p1 = (imm >> 12) & 0b1111_1111;
    let p2 = (imm >> 11) & 1;
    let p3 = (imm >> 1)  & 0b11111_11111;
    let p4 = (imm >> 20) & 1;
    ( ((p4 << 18) | (p3 << 9) | (p2 << 8) | p1) << 1 ) as u32
}




// Pseudoinstructions implementation

#[derive(Debug, Copy, Clone)]
pub enum PseudoInstruction {
    LI,
    RET,
    MV,
    LA,
}

impl Pseudo for PseudoInstruction {
    fn translate(&self, args: Vec<ArgValue>) -> Vec<(Box<dyn Extension>, Vec<ArgValue>)>  {
        let mut v: Vec<(Box<dyn Extension>, Vec<ArgValue>)> = vec![];
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

                if let (ArgValue::REGISTER(_), ArgValue::NUMBER(n)) = (arg1, arg2) {
                    let mut args: Vec<ArgValue> = Vec::new();
                    let upper20bits: i32 = (n >> 12) & 0b11111_11111_11111_11111;
                    let lower12bits: i32 = n & 0b1111_1111_1111;

                    if (*n >= -2048) && (*n <= 2047) {
                        //If the immediate fits in a signed 12-bit immediate, then 'li' gets
                        //simplified to a 'addi' op
                        args.push(arg1.clone());
                        args.push(ArgValue::REGISTER(Register::X0));
                        args.push(ArgValue::NUMBER(lower12bits));
                        v.push((Box::new(RV32I::ADDI), args.drain(..).collect()));
                    }
                    else {
                        //Otherwise we have to load the upper 20 bits of the immediate using
                        //'LUI', followed by a 'addi' op
                        args.push(arg1.clone());
                        args.push(ArgValue::NUMBER(upper20bits));
                        v.push((Box::new(RV32I::LUI), args.drain(..).collect()));

                        args.clear();
                        args.push(arg1.clone());
                        args.push(arg1.clone());
                        args.push(ArgValue::NUMBER(lower12bits));
                        v.push((Box::new(RV32I::ADDI), args.drain(..).collect()));
                    }
                }
            },

            //ret, becomes:
            //jalr x0 x1 0
            PseudoInstruction::RET => {
                let mut args = Vec::new();

                args.push(ArgValue::REGISTER(Register::X0));
                args.push(ArgValue::REGISTER(Register::X1));
                args.push(ArgValue::NUMBER(0));

                v.push((Box::new(RV32I::JALR), args));
            },

            //mv rd, rs1, becomes:
            //addi rd, rs1, 0
            PseudoInstruction::MV => {
                let arg1: ArgValue = args.get(0).unwrap().clone();
                let arg2: ArgValue = args.get(1).unwrap().clone();
                let mut args = Vec::new();
                args.push(arg1);
                args.push(arg2);
                args.push(ArgValue::NUMBER(0));

                v.push((Box::new(RV32I::ADDI), args));
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
                if let (ArgValue::REGISTER(_), ArgValue::NUMBER(n)) = (arg1, arg2) {
                    let mut args: Vec<ArgValue> = Vec::new();
                    let upper20bits: i32 = (n >> 12) & 0b11111_11111_11111_11111;
                    let lower12bits: i32 = n & 0b1111_1111_1111;

                    if (*n >= -2048) && (*n <= 2047) {
                        args.push(arg1.clone());
                        args.push(ArgValue::REGISTER(Register::X0));
                        args.push(ArgValue::NUMBER(lower12bits));
                        v.push((Box::new(RV32I::ADDI), args.drain(..).collect()));
                    }
                    else {
                        args.push(arg1.clone());
                        args.push(ArgValue::NUMBER(upper20bits));
                        v.push((Box::new(RV32I::AUIPC), args.drain(..).collect()));

                        args.clear();
                        args.push(arg1.clone());
                        args.push(arg1.clone());
                        args.push(ArgValue::NUMBER(lower12bits));
                        v.push((Box::new(RV32I::ADDI), args.drain(..).collect()));
                    }
                }
                else if let (ArgValue::REGISTER(_), ArgValue::USE(s)) = (arg1, arg2) {
                    let mut args: Vec<ArgValue> = Vec::new();
                    let upper20bits = ArgValue::USEHI(s.to_string());
                    let lower12bits = ArgValue::USELO(s.to_string());
                    //We can't know if HI is 0 or not, therefore we can't optimize
                    args.push(arg1.clone());
                    args.push(upper20bits);
                    v.push((Box::new(RV32I::AUIPC), args.drain(..).collect()));
                    args.clear();
                    args.push(arg1.clone());
                    args.push(arg1.clone());
                    args.push(lower12bits);
                    v.push((Box::new(RV32I::ADDI), args.drain(..).collect()));
                }
            }
        }
        v
    }
}




// Directives implementation

#[derive(Debug)]
pub enum DirectiveInstruction {
    WORD,
    HALF,
    BYTE,
    SKIP,
    ASCII,
    GLOBL,
}

impl Directive for DirectiveInstruction {
    fn translate(&self, args: &Vec<ArgValue>) -> Vec<u8>  {
        let mut v = Vec::new();
        match self {
            DirectiveInstruction::WORD => {
                let arg = &args[0];
                match arg {
                    ArgValue::NUMBER(n) => {
                        v.push(((n & 0b00000000_00000000_00000000_11111111) >> 0).try_into().unwrap());
                        v.push(((n & 0b00000000_00000000_11111111_00000000) >> 8).try_into().unwrap());
                        v.push(((n & 0b00000000_11111111_00000000_00000000) >> 16).try_into().unwrap());
                        v.push(((n & 0b11111111_00000000_00000000_00000000u32 as i32) >> 24).try_into().unwrap());
                    },
                    _ => panic!(),
                }
            },
            DirectiveInstruction::SKIP => {
                let arg = &args[0];
                match arg {
                    ArgValue::NUMBER(n) => {
                        let capacity: usize = (*n).try_into().unwrap();
                        v.reserve(capacity + capacity % 4);
                    },
                    _ => panic!(),
                }
            },
            DirectiveInstruction::ASCII => {
                let arg = &args[0];
                match arg {
                    ArgValue::LITERAL(s) => {
                        let lits: Vec<u8> = s.bytes().collect();
                        v.extend(lits);
                    },
                    _ => panic!(),
                }
            },
            DirectiveInstruction::GLOBL => {
            },
            DirectiveInstruction::HALF => {
            },
            DirectiveInstruction::BYTE => {

            },
        }
        v
    }
}
