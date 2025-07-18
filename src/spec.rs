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



// Available Instruction Binary Formats (as in the ISA)

#[derive(Debug, Copy, Clone)]
pub enum InstructionFormat {
    R{funct7: i32, rs2: i32, rs1: i32, funct3: i32, rd: i32, opcode: i32},
    I{imm: i32, rs1: i32, funct3: i32, rd: i32, opcode: i32},
    S{imm1: i32, rs2: i32, rs1: i32, funct3: i32, imm2: i32, opcode: i32},
    B{imm1: i32, rs2: i32, rs1: i32, funct3: i32, imm2: i32, opcode: i32},
    U{imm: i32, rd: i32, opcode: i32},
    J{imm: i32, rd: i32, opcode: i32},
}

impl InstructionFormat {
    pub fn encode(&self) -> u32 {
        match self {
            InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let rs1    = cast_5bits(rs1);
                let rs2    = cast_5bits(rs2);
                let funct3 = cast_3bits(funct3);
                let funct7 = cast_7bits(funct7);
                (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            InstructionFormat::I { imm, rs1, funct3, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let rs1    = cast_5bits(rs1);
                let funct3 = cast_3bits(funct3);
                let imm    = cast_12bits(imm);
                (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
            },
            InstructionFormat::S { imm1, rs2, rs1, funct3, imm2, opcode } => {
                let opcode = cast_7bits(opcode);
                let rs1    = cast_5bits(rs1);
                let rs2    = cast_5bits(rs2);
                let funct3 = cast_3bits(funct3);
                let imm2   = cast_5bits(imm2);
                let imm1   = cast_7bits(imm1);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            InstructionFormat::B { imm1, rs2, rs1, funct3, imm2, opcode } => {
                let opcode = cast_7bits(opcode);
                let rs1    = cast_5bits(rs1);
                let rs2    = cast_5bits(rs2);
                let funct3 = cast_3bits(funct3);
                let imm2   = cast_5bits(imm2);
                let imm1   = cast_7bits(imm1);
                (imm1 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm2 << 7) | opcode
            },
            InstructionFormat::U { imm, rd, opcode } => {
                let opcode = cast_7bits(opcode);
                let rd     = cast_5bits(rd);
                let imm    = cast_20bits(imm);
                (imm << 12) | (rd << 7) | opcode
            },
            InstructionFormat::J { imm, rd, opcode } => {
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



// Instruction Assembly Description

pub enum ArgKey {
    RS1,
    RS2,
    RD,
    IMM,
    OFF,
}

#[derive(Debug, Clone, Copy)]
pub enum ArgValue {
    NUMBER(i32),
    REG(i32),
}

pub enum ArgSyntax {
    N0,
    N1(ArgKey),
    N2(ArgKey, ArgKey),
    N3(ArgKey, ArgKey, ArgKey),
    N4(ArgKey, ArgKey, ArgKey, ArgKey),
}

pub trait ToArg {
    type Token;
    fn to_arg(&self, token: Self::Token) -> Option<ArgValue> ;
}

pub fn instruction_to_binary(inst: &Box<dyn Extension>, args: &Vec<ArgValue>) -> u32 {
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
    fields: Vec<ArgKey>,
    args: &Vec<ArgValue>
) -> (i32, i32, i32, i32)
{
    let mut rs1: i32 = 0;
    let mut rs2: i32 = 0;
    let mut rd: i32 = 0;
    let mut imm: i32 = 0;
    for (field, arg) in fields.iter().zip(args.iter()) {
        match arg {
            ArgValue::NUMBER(v) => imm = *v,
            ArgValue::REG(reg) => {
                match field {
                    ArgKey::RS1 => rs1 = *reg,
                    ArgKey::RS2 => rs2 = *reg,
                    ArgKey::RD => rd = *reg,
                    _ => eprintln!("Error")
                }
            },
        }
    }
    (rs1, rs2, rd, imm)
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
    fn get_instruction_format(&self, rs1: i32, rs2: i32, rd: i32, imm: i32) -> InstructionFormat ;
    fn get_calling_syntax(&self) -> ArgSyntax ;
    fn clone_box(&self) -> Box<dyn Extension> ;
}

impl Clone for Box<dyn Extension> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
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
    fn get_instruction_format(&self, rs1: i32, rs2: i32, rd: i32, imm: i32) -> InstructionFormat  {
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
            RV32I::JALR  => InstructionFormat::I { imm: imm_to_i(imm), rs1, funct3: 0b000, rd, opcode: 0b1101111 },
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
            RV32I::ADD   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::SUB   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::AND   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::OR    => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::XOR   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::SLL   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::SRL   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::SRA   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::SLT   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::SLTU  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::RS2),
            RV32I::LUI   => ArgSyntax::N2(ArgKey::RD, ArgKey::IMM),
            RV32I::AUIPC => ArgSyntax::N2(ArgKey::RD, ArgKey::IMM),
            RV32I::JAL   => ArgSyntax::N2(ArgKey::RD, ArgKey::OFF),
            RV32I::JALR  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::OFF),
            RV32I::ADDI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::ANDI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::ORI   => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::XORI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::SLTI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::SLTIU => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::SLLI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::SRLI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::SRAI  => ArgSyntax::N3(ArgKey::RD, ArgKey::RS1, ArgKey::IMM),
            RV32I::LW    => ArgSyntax::N3(ArgKey::RD, ArgKey::OFF, ArgKey::RS1),
            RV32I::LH    => ArgSyntax::N3(ArgKey::RD, ArgKey::OFF, ArgKey::RS1),
            RV32I::LB    => ArgSyntax::N3(ArgKey::RD, ArgKey::OFF, ArgKey::RS1),
            RV32I::LHU   => ArgSyntax::N3(ArgKey::RD, ArgKey::OFF, ArgKey::RS1),
            RV32I::LBU   => ArgSyntax::N3(ArgKey::RD, ArgKey::OFF, ArgKey::RS1),
            RV32I::SW    => ArgSyntax::N3(ArgKey::RS2, ArgKey::OFF, ArgKey::RS1),
            RV32I::SH    => ArgSyntax::N3(ArgKey::RS2, ArgKey::OFF, ArgKey::RS1),
            RV32I::SB    => ArgSyntax::N3(ArgKey::RS2, ArgKey::OFF, ArgKey::RS1),
            RV32I::BEQ   => ArgSyntax::N3(ArgKey::RS1, ArgKey::RS2, ArgKey::OFF),
            RV32I::BNE   => ArgSyntax::N3(ArgKey::RS1, ArgKey::RS2, ArgKey::OFF),
            RV32I::BLT   => ArgSyntax::N3(ArgKey::RS1, ArgKey::RS2, ArgKey::OFF),
            RV32I::BLTU  => ArgSyntax::N3(ArgKey::RS1, ArgKey::RS2, ArgKey::OFF),
            RV32I::BGE   => ArgSyntax::N3(ArgKey::RS1, ArgKey::RS2, ArgKey::OFF),
            RV32I::BGEU  => ArgSyntax::N3(ArgKey::RS1, ArgKey::RS2, ArgKey::OFF),
            RV32I::FENCE => todo!(),
        }
    }

    fn clone_box(&self) -> Box<dyn Extension>  {
        Box::new(self.clone())
    }
}

//convert an immediate as read from the parser into the number to be stored in an Instruction.
//the Instruction is going to use this result as-is later to assemble a 32-bit instruction.

//TODO: handle sign extension
//TODO: This could later be turned into a struct/enum like 'InstructionImmediate' or just 'Immediate'

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
    // (imm >> 12) & 0b11111_11111_11111_11111
    imm & 0b11111_11111_11111_11111
}

fn imm_to_j(imm: i32) -> i32 {
    let p1 = (imm >> 12) & 0b1111_1111;
    let p2 = (imm >> 11) & 1;
    let p3 = (imm >> 1)  & 0b11111_11111;
    let p4 = (imm >> 20) & 1;
    ((p4 << 18) | (p3 << 9) | (p2 << 8) | p1) << 1
}
