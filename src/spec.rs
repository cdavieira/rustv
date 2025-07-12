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

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    R{funct7: i32, rs2: i32, rs1: i32, funct3: i32, rd: i32, opcode: i32},
    I{imm: i32, rs1: i32, funct3: i32, rd: i32, opcode: i32},
    S{imm1: i32, rs2: i32, rs1: i32, funct3: i32, imm2: i32, opcode: i32},
    B{imm1: i32, rs2: i32, rs1: i32, funct3: i32, imm2: i32, opcode: i32},
    U{imm: i32, rd: i32, opcode: i32},
    J{imm: i32, rd: i32, opcode: i32},
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
    fn get_format(&self) -> Instruction ;
    fn get_bytes(&self, rs1: i32, rs2: i32, rd: i32, imm1: i32, imm2: i32) -> u32 ;
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
    fn get_format(&self) -> Instruction  {
        match self {
            RV32I::ADD   => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b000, rd: 0, opcode: 0b0110011 },
            RV32I::SUB   => Instruction::R { funct7: 0b1000000, rs2: 0, rs1: 0, funct3: 0b000, rd: 0, opcode: 0b0110011 },
            RV32I::AND   => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b111, rd: 0, opcode: 0b0110011 },
            RV32I::OR    => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b110, rd: 0, opcode: 0b0110011 },
            RV32I::XOR   => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b100, rd: 0, opcode: 0b0110011 },
            RV32I::SLL   => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b001, rd: 0, opcode: 0b0110011 },
            RV32I::SRL   => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b101, rd: 0, opcode: 0b0110011 },
            RV32I::SRA   => Instruction::R { funct7: 0b0100000, rs2: 0, rs1: 0, funct3: 0b101, rd: 0, opcode: 0b0110011 },
            RV32I::SLT   => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b010, rd: 0, opcode: 0b0110011 },
            RV32I::SLTU  => Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b011, rd: 0, opcode: 0b0110011 },
            RV32I::LUI   => Instruction::U { imm: 0, rd: 0, opcode: 0b01101 },
            RV32I::AUIPC => Instruction::U { imm: 0, rd: 0, opcode: 0b00101 },
            RV32I::JAL   => Instruction::J { imm: 0, rd: 0, opcode: 0b1101111 },
            RV32I::JALR  => Instruction::I { imm: 0, rs1: 0, funct3: 0b000, rd: 0, opcode: 0b1101111 },
            RV32I::ADDI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b000, rd: 0, opcode: 0b0010011 },
            RV32I::ANDI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b111, rd: 0, opcode: 0b0010011 },
            RV32I::ORI   => Instruction::I { imm: 0, rs1: 0, funct3: 0b110, rd: 0, opcode: 0b0010011 },
            RV32I::XORI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b100, rd: 0, opcode: 0b0010011 },
            RV32I::SLTI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b010, rd: 0, opcode: 0b0010011 },
            RV32I::SLTIU => Instruction::I { imm: 0, rs1: 0, funct3: 0b011, rd: 0, opcode: 0b0010011 },
            RV32I::SLLI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b001, rd: 0, opcode: 0b0010011 },
            RV32I::SRLI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b101, rd: 0, opcode: 0b0010011 },
            RV32I::SRAI  => Instruction::I { imm: 0, rs1: 0, funct3: 0b101, rd: 0, opcode: 0b0010011 },
            RV32I::LW    => Instruction::I { imm: 0, rs1: 0, funct3: 0b010, rd: 0, opcode: 0b0000011 },
            RV32I::LH    => Instruction::I { imm: 0, rs1: 0, funct3: 0b001, rd: 0, opcode: 0b0000011 },
            RV32I::LB    => Instruction::I { imm: 0, rs1: 0, funct3: 0b000, rd: 0, opcode: 0b0000011 },
            RV32I::LHU   => Instruction::I { imm: 0, rs1: 0, funct3: 0b101, rd: 0, opcode: 0b0000011 },
            RV32I::LBU   => Instruction::I { imm: 0, rs1: 0, funct3: 0b100, rd: 0, opcode: 0b0000011 },
            RV32I::SW    => Instruction::S { imm1: 0, rs2: 0, rs1: 0, funct3: 0b010, imm2: 0, opcode: 0b0100011 },
            RV32I::SH    => Instruction::S { imm1: 0, rs2: 0, rs1: 0, funct3: 0b001, imm2: 0, opcode: 0b0100011 },
            RV32I::SB    => Instruction::S { imm1: 0, rs2: 0, rs1: 0, funct3: 0b000, imm2: 0, opcode: 0b0100011 },
            RV32I::BEQ   => Instruction::B { imm1: 0, rs2: 0, rs1: 0, funct3: 0b000, imm2: 0, opcode: 0b1100011 },
            RV32I::BNE   => Instruction::B { imm1: 0, rs2: 0, rs1: 0, funct3: 0b001, imm2: 0, opcode: 0b1100011 },
            RV32I::BLT   => Instruction::B { imm1: 0, rs2: 0, rs1: 0, funct3: 0b100, imm2: 0, opcode: 0b1100011 },
            RV32I::BLTU  => Instruction::B { imm1: 0, rs2: 0, rs1: 0, funct3: 0b110, imm2: 0, opcode: 0b1100011 },
            RV32I::BGE   => Instruction::B { imm1: 0, rs2: 0, rs1: 0, funct3: 0b101, imm2: 0, opcode: 0b1100011 },
            RV32I::BGEU  => Instruction::B { imm1: 0, rs2: 0, rs1: 0, funct3: 0b111, imm2: 0, opcode: 0b1100011 },
            RV32I::FENCE => todo!(),
        }
    }

    //add, and, or (R), lui (U), jal (J), addi, andi, ori, lw (I), sw (S), beq, blt (B)
    //TODO: handle sign extension
    //TODO: handle SLLI, SRLI, SRLA differently (I)
    fn get_bytes(&self, rs1: i32, rs2: i32, rd: i32, imm1: i32, imm2: i32) -> u32 {
        match self.get_format() {
            Instruction::R { funct7, rs2: _, rs1: _, funct3, rd: _, opcode } => {
                let opcode: u32 = (opcode & 0b1_111_111).try_into().unwrap();
                let rd    : u32 = (rd & 0b111_11).try_into().unwrap();
                let funct3: u32 = (funct3 & 0b111).try_into().unwrap();
                let rs1   : u32 = (rs1 & 0b11_111).try_into().unwrap();
                let rs2   : u32 = (rs2 & 0b11_111).try_into().unwrap();
                let funct7: u32 = (funct7 & 0b1_111_111).try_into().unwrap();
                (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            Instruction::I { imm: _, rs1: _, funct3, rd: _, opcode }         => {
                let opcode: u32 = (opcode & 0b1_111_111).try_into().unwrap();
                let rd    : u32 = (rd & 0b111_11).try_into().unwrap();
                let funct3: u32 = (funct3 & 0b111).try_into().unwrap();
                let rs1   : u32 = (rs1 & 0b11_111).try_into().unwrap();
                let imm   : u32 = (imm1 & 0b111_111_111_111).try_into().unwrap();
                (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            Instruction::S { imm1: _, rs2: _, rs1: _, funct3, imm2: _, opcode }        => {
                let opcode: u32 = (opcode & 0b1_111_111).try_into().unwrap();
                let imm2  : u32 = (imm2 & 0b111_11).try_into().unwrap();
                let funct3: u32 = (funct3 & 0b111).try_into().unwrap();
                let rs1   : u32 = (rs1 & 0b11_111).try_into().unwrap();
                let rs2   : u32 = (rs2 & 0b11_111).try_into().unwrap();
                let imm1  : u32 = (imm1 & 0b1_111_111).try_into().unwrap();
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            Instruction::B { imm1: _, rs2: _, rs1: _, funct3, imm2: _, opcode }        => {
                let opcode: u32 = (opcode & 0b1_111_111).try_into().unwrap();
                let imm2  : u32 = (imm2 & 0b111_11).try_into().unwrap();
                let funct3: u32 = (funct3 & 0b111).try_into().unwrap();
                let rs1   : u32 = (rs1 & 0b11_111).try_into().unwrap();
                let rs2   : u32 = (rs2 & 0b11_111).try_into().unwrap();
                let imm1  : u32 = (imm1 & 0b1_111_111).try_into().unwrap();
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            Instruction::U { imm: _, rd: _, opcode } => {
                let opcode: u32 = (opcode & 0b1_111_111).try_into().unwrap();
                let rd    : u32 = (rd & 0b111_11).try_into().unwrap();
                let imm   : u32 = (imm1 & 0b11111_11111_11111_11111).try_into().unwrap();
                (imm << 12) | (rd << 7) | opcode
            },
            Instruction::J { imm: _, rd: _, opcode } => {
                let opcode: u32 = (opcode & 0b1_111_111).try_into().unwrap();
                let rd    : u32 = (rd & 0b111_11).try_into().unwrap();
                let imm   : u32 = (imm1 & 0b11111_11111_11111_11111).try_into().unwrap();
                (imm << 12) | (rd << 7) | opcode
            },
        }
    }
}

// pub enum RV64I {
//     AAA
// }
// impl Extension for RV64I {
//     fn get_format(&self) -> Instruction  {
//         Instruction::R { funct7: 0b0000000, rs2: 0, rs1: 0, funct3: 0b000, rd: 0, opcode: 0b0110011 }
//     }
// }
