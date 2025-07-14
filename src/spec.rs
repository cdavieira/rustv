// Registers

#[derive(Debug, Copy, Clone)]
pub enum Register {
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15, X16, X17, X18, X19, X20,
    X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, X31, PC,
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



// Instruction Formats

pub enum Field {
    RS1,
    RS2,
    RD,
    IMM,
    OFF,
}

pub enum Syntax {
    N0,
    N1(Field),
    N2(Field, Field),
    N3(Field, Field, Field),
    N4(Field, Field, Field, Field),
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    R{funct7: i32, rs2: i32, rs1: i32, funct3: i32, rd: i32, opcode: i32},
    I{imm: i32, rs1: i32, funct3: i32, rd: i32, opcode: i32},
    S{imm1: i32, rs2: i32, rs1: i32, funct3: i32, imm2: i32, opcode: i32},
    B{imm1: i32, rs2: i32, rs1: i32, funct3: i32, imm2: i32, opcode: i32},
    U{imm: i32, rd: i32, opcode: i32},
    J{imm: i32, rd: i32, opcode: i32},
}

impl Instruction {
    pub fn get_bytes(&self) -> u32 {
        match self {
            Instruction::R { funct7, rs2, rs1, funct3, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let rs1    = cast_5bits(rs1);
                let rs2    = cast_5bits(rs2);
                let funct3 = cast_3bits(funct3);
                let funct7 = cast_7bits(funct7);
                (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            Instruction::I { imm, rs1, funct3, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let rs1    = cast_5bits(rs1);
                let funct3 = cast_3bits(funct3);
                let imm    = cast_12bits(imm);
                (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            Instruction::S { imm1, rs2, rs1, funct3, imm2, opcode } => {
                let opcode = cast_7bits(opcode);
                let rs1    = cast_5bits(rs1);
                let rs2    = cast_5bits(rs2);
                let funct3 = cast_3bits(funct3);
                let imm2   = cast_5bits(imm2);
                let imm1   = cast_7bits(imm1);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            Instruction::B { imm1, rs2, rs1, funct3, imm2, opcode } => {
                let opcode = cast_7bits(opcode);
                let rs1    = cast_5bits(rs1);
                let rs2    = cast_5bits(rs2);
                let funct3 = cast_3bits(funct3);
                let imm2   = cast_5bits(imm2);
                let imm1   = cast_7bits(imm1);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            Instruction::U { imm, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let imm    = cast_20bits(imm);
                (imm << 12) | (rd << 7) | opcode
            },
            Instruction::J { imm, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let imm    = cast_20bits(imm);
                (imm << 12) | (rd << 7) | opcode
            },
        }
    }
}

fn cast_3bits(f3: &i32) -> u32 {
    (f3 & 0b111).try_into().unwrap()
}

fn cast_5bits(reg: &i32) -> u32 {
    (reg & 0b11111).try_into().unwrap()
}

fn cast_7bits(f7: &i32) -> u32 {
    (f7 & 0b1_111_111).try_into().unwrap()
}

fn cast_12bits(f7: &i32) -> u32 {
    (f7 & 0b1111_1111_1111).try_into().unwrap()
}

fn cast_20bits(f7: &i32) -> u32 {
    (f7 & 0b11111_11111_11111_11111).try_into().unwrap()
}



// Extensions

/** 
An extension was thought as a set of new instructions which can extend the functionalities
offered by the assembly language to access and interact with the hardware.

Each new instruction of the extension must have a format (as described in the riscv
specification), which can be found in the enum 'Instruction'

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
    fn get_instruction(&self, rs1: i32, rs2: i32, rd: i32, imm: i32) -> Instruction ;
    fn get_syntax(&self) -> Syntax ;
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
#[derive(Debug, Copy, Clone)]
pub enum RV32I {
    LUI, AUIPC, ADDI, ANDI, ORI, XORI, ADD, SUB, AND, OR, XOR, SLL, SRL, SRA, FENCE, SLTI, SLTIU,
    SLLI, SRLI, SRAI, SLT, SLTU, LW, LH, LHU, LB, LBU, SW, SH, SB, JAL, JALR, BEQ, BNE, BLT, BLTU,
    BGE, BGEU,
}

impl Extension for RV32I {
    fn get_instruction(&self, rs1: i32, rs2: i32, rd: i32, imm: i32) -> Instruction  {
        match self {
            RV32I::ADD   => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b000, rd, opcode: 0b0110011 },
            RV32I::SUB   => Instruction::R { funct7: 0b1000000, rs2, rs1, funct3: 0b000, rd, opcode: 0b0110011 },
            RV32I::AND   => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b111, rd, opcode: 0b0110011 },
            RV32I::OR    => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b110, rd, opcode: 0b0110011 },
            RV32I::XOR   => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b100, rd, opcode: 0b0110011 },
            RV32I::SLL   => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b001, rd, opcode: 0b0110011 },
            RV32I::SRL   => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b101, rd, opcode: 0b0110011 },
            RV32I::SRA   => Instruction::R { funct7: 0b0100000, rs2, rs1, funct3: 0b101, rd, opcode: 0b0110011 },
            RV32I::SLT   => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b010, rd, opcode: 0b0110011 },
            RV32I::SLTU  => Instruction::R { funct7: 0b0000000, rs2, rs1, funct3: 0b011, rd, opcode: 0b0110011 },
            RV32I::LUI   => Instruction::U { imm: imm_to_u(imm), rd, opcode: 0b0110111 },
            RV32I::AUIPC => Instruction::U { imm: imm_to_u(imm), rd, opcode: 0b0010111 },
            RV32I::JAL   => Instruction::J { imm: imm_to_j(imm), rd, opcode: 0b1101111 },
            RV32I::JALR  => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b1101111 },
            RV32I::ADDI  => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b0010011 },
            RV32I::ANDI  => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b111, rd, opcode: 0b0010011 },
            RV32I::ORI   => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b110, rd, opcode: 0b0010011 },
            RV32I::XORI  => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b100, rd, opcode: 0b0010011 },
            RV32I::SLTI  => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b010, rd, opcode: 0b0010011 },
            RV32I::SLTIU => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b011, rd, opcode: 0b0010011 },
            RV32I::SLLI  => Instruction::I { imm: imm_to_i(0b00_00000_11111 & imm), rs1, funct3: 0b001, rd, opcode: 0b0010011 },
            RV32I::SRLI  => Instruction::I { imm: imm_to_i(0b00_00000_11111 & imm), rs1, funct3: 0b101, rd, opcode: 0b0010011 },
            RV32I::SRAI  => Instruction::I { imm: imm_to_i(0b01_00000_11111 & imm), rs1, funct3: 0b101, rd, opcode: 0b0010011 },
            RV32I::LW    => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b010, rd, opcode: 0b0000011 },
            RV32I::LH    => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b001, rd, opcode: 0b0000011 },
            RV32I::LB    => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b0000011 },
            RV32I::LHU   => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b101, rd, opcode: 0b0000011 },
            RV32I::LBU   => Instruction::I { imm: imm_to_i(imm), rs1, funct3: 0b100, rd, opcode: 0b0000011 },
            RV32I::SW    => {
                let (imm1, imm2) = imm_to_s(imm);
                Instruction::S { imm1, rs2, rs1, funct3: 0b010, imm2, opcode: 0b0100011 }
            },
            RV32I::SH    => {
                let (imm1, imm2) = imm_to_s(imm);
                Instruction::S { imm1, rs2, rs1, funct3: 0b001, imm2, opcode: 0b0100011 }
            },
            RV32I::SB    => {
                let (imm1, imm2) = imm_to_s(imm);
                Instruction::S { imm1, rs2, rs1, funct3: 0b000, imm2, opcode: 0b0100011 }
            },
            RV32I::BEQ   => {
                let (imm1, imm2) = imm_to_b(imm);
                Instruction::B { imm1, rs2, rs1, funct3: 0b000, imm2, opcode: 0b1100011 }
            },
            RV32I::BNE   => {
                let (imm1, imm2) = imm_to_b(imm);
                Instruction::B { imm1, rs2, rs1, funct3: 0b001, imm2, opcode: 0b1100011 }
            },
            RV32I::BLT   => {
                let (imm1, imm2) = imm_to_b(imm);
                Instruction::B { imm1, rs2, rs1, funct3: 0b100, imm2, opcode: 0b1100011 }
            },
            RV32I::BLTU  => {
                let (imm1, imm2) = imm_to_b(imm);
                Instruction::B { imm1, rs2, rs1, funct3: 0b110, imm2, opcode: 0b1100011 }
            },
            RV32I::BGE   => {
                let (imm1, imm2) = imm_to_b(imm);
                Instruction::B { imm1, rs2, rs1, funct3: 0b101, imm2, opcode: 0b1100011 }
            },
            RV32I::BGEU  => {
                let (imm1, imm2) = imm_to_b(imm);
                Instruction::B { imm1, rs2, rs1, funct3: 0b111, imm2, opcode: 0b1100011 }
            },
            RV32I::FENCE => todo!(),
        }
    }

    fn get_syntax(&self) -> Syntax {
        match self {
            RV32I::ADD   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::SUB   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::AND   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::OR    => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::XOR   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::SLL   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::SRL   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::SRA   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::SLT   => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::SLTU  => Syntax::N3(Field::RD, Field::RS1, Field::RS2),
            RV32I::LUI   => Syntax::N2(Field::RD, Field::IMM),
            RV32I::AUIPC => Syntax::N2(Field::RD, Field::IMM),
            RV32I::JAL   => Syntax::N2(Field::RD, Field::OFF),
            RV32I::JALR  => Syntax::N3(Field::RD, Field::RS1, Field::OFF),
            RV32I::ADDI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::ANDI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::ORI   => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::XORI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::SLTI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::SLTIU => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::SLLI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::SRLI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::SRAI  => Syntax::N3(Field::RD, Field::RS1, Field::IMM),
            RV32I::LW    => Syntax::N3(Field::RD, Field::OFF, Field::RS1),
            RV32I::LH    => Syntax::N3(Field::RD, Field::OFF, Field::RS1),
            RV32I::LB    => Syntax::N3(Field::RD, Field::OFF, Field::RS1),
            RV32I::LHU   => Syntax::N3(Field::RD, Field::OFF, Field::RS1),
            RV32I::LBU   => Syntax::N3(Field::RD, Field::OFF, Field::RS1),
            RV32I::SW    => Syntax::N3(Field::RS2, Field::OFF, Field::RS1),
            RV32I::SH    => Syntax::N3(Field::RS2, Field::OFF, Field::RS1),
            RV32I::SB    => Syntax::N3(Field::RS2, Field::OFF, Field::RS1),
            RV32I::BEQ   => Syntax::N3(Field::RS1, Field::RS2, Field::OFF),
            RV32I::BNE   => Syntax::N3(Field::RS1, Field::RS2, Field::OFF),
            RV32I::BLT   => Syntax::N3(Field::RS1, Field::RS2, Field::OFF),
            RV32I::BLTU  => Syntax::N3(Field::RS1, Field::RS2, Field::OFF),
            RV32I::BGE   => Syntax::N3(Field::RS1, Field::RS2, Field::OFF),
            RV32I::BGEU  => Syntax::N3(Field::RS1, Field::RS2, Field::OFF),
            RV32I::FENCE => todo!(),
        }
    }
}

//convert an immediate as read from the parser into the number to be stored in an Instruction.
//the Instruction is going to use this result as-is later to assemble a 32-bit instruction.

//TODO: handle sign extension

fn imm_to_i(imm: i32) -> i32 {
    imm & 0b1111_1111_1111
}

fn imm_to_s(imm: i32) -> (i32, i32) {
    let imm1 = (imm & 0b1111_111_00000) >> 5;
    let imm2 = imm & 0b11111;
    (imm1, imm2)
}

fn imm_to_b(imm: i32) -> (i32, i32) {
    let bit12 = imm & 0b100_000_000_000;
    let bit13 = imm & 0b1_000_000_000_000;
    let imm1 = ((imm & 0b111111_00000) >> 5) | (bit13 >> 6);
    let imm2 = (imm & 0b11110) | (bit12 >> 11);
    (imm1, imm2)
}

fn imm_to_u(imm: i32) -> i32 {
    (imm >> 12) & 0b11111_11111_11111_11111
}

fn imm_to_j(imm: i32) -> i32 {
    let p1 = (imm >> 12) & 0b1111_1111;
    let p2 = (imm >> 11) & 1;
    let p3 = (imm >> 1)  & 0b11111_11111;
    let p4 = (imm >> 20) & 1;
    ((p4 << 18) | (p3 << 9) | (p2 << 8) | p1) << 1
}



//Pseudo Instruction
pub struct PseudoInstruction {
}
