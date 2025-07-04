#[derive(Debug, Copy, Clone)]

pub enum Register {
    X0, X1, X2, X3, X4, X5,
    X6, X7, X8, X9, X10, X11,
    X12, X13, X14, X15, X16, X17,
    X18, X19, X20, X21, X22, X23,
    X24, X25, X26, X27, X28, X29,
    X30, X31, PC,
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




//all extensions should go here

trait Extension {
    type Opcode;
    type Token;

    fn str_to_opcode(&self, s: &str) -> Option<Self::Opcode> {
        None
    }

    fn opcode_to_token(&self) -> Option<Self::Token> {
        None
    }
}

pub mod extensions {
    pub mod rv32i {
        #[derive(Debug, Copy, Clone)]
        pub enum Opcode {
            LUI,
            AUIPC,
            ADDI,
            ANDI,
            ORI,
            XORI,
            ADD,
            SUB,
            AND,
            OR,
            XOR,
            SLL,
            SRL,
            SRA,
            FENCE,
            SLTI,
            SLTIU,
            SLLI,
            SRLI,
            SRAI,
            SLT,
            SLTU,
            LW,
            SW,
        }
    }
}
