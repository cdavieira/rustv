use crate::lang::{
    directive::Directive,
    ext::Extension,
    pseudo::Pseudo,
};



#[derive(Debug, Copy, Clone)]
pub enum Register {
    // Numerical variants
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15,
    X16, X17, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, X31,

    // PC
    PC,

    // Semantic names for the numerical variants
    ZERO, // Null Register
    RA,   // Return Address
    SP,   // Stack Pointer
    GP,   // Global Pointer
    TP,   // Thread Pointer
    FP, S0, // Saved Register/Frame Pointer
    S1,   // Saved Register
    A0, A1, // Function Arguments/Return Values
    A2, A3, A4, A5, A6, A7, // Function Arguments
    S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, // Saved Registers
    T0, T1, T2, T3, T4, T5, T6, // Temporaries
}

impl Register {
    pub fn id(&self) -> u8 {
        match self {
            Register::X0  | Register::ZERO => 0,
            Register::X1  | Register::RA   => 1,
            Register::X2  | Register::SP   => 2,
            Register::X3  | Register::GP   => 3,
            Register::X4  | Register::TP   => 4,
            Register::X5  | Register::T0   => 5,
            Register::X6  | Register::T1   => 6,
            Register::X7  | Register::T2   => 7,
            Register::X8  | Register::S0 | Register::FP => 8,
            Register::X9  | Register::S1   => 9,
            Register::X10 | Register::A0   => 10,
            Register::X11 | Register::A1   => 11,
            Register::X12 | Register::A2   => 12,
            Register::X13 | Register::A3   => 13,
            Register::X14 | Register::A4   => 14,
            Register::X15 | Register::A5   => 15,
            Register::X16 | Register::A6   => 16,
            Register::X17 | Register::A7   => 17,
            Register::X18 | Register::S2   => 18,
            Register::X19 | Register::S3   => 19,
            Register::X20 | Register::S4   => 20,
            Register::X21 | Register::S5   => 21,
            Register::X22 | Register::S6   => 22,
            Register::X23 | Register::S7   => 23,
            Register::X24 | Register::S8   => 24,
            Register::X25 | Register::S9   => 25,
            Register::X26 | Register::S10  => 26,
            Register::X27 | Register::S11  => 27,
            Register::X28 | Register::T3   => 28,
            Register::X29 | Register::T4   => 29,
            Register::X30 | Register::T5   => 30,
            Register::X31 | Register::T6   => 31,
            Register::PC  => todo!(),
        }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum SectionName {
    Metadata,
    Text,
    Data,
    Bss,
    Custom(String)
}

impl SectionName {
    pub fn default_name(&self) -> String {
        match self {
            SectionName::Metadata  => String::from(".meta"),
            SectionName::Text      => String::from(".text"),
            SectionName::Data      => String::from(".data"),
            SectionName::Bss       => String::from(".bss"),
            SectionName::Custom(s) => s.to_string(),
        }
    }
    
}




pub enum Datatype {
    Word,
    Half,
    Byte,
    Ascii,
}

impl Datatype {
    pub fn alignment(&self) -> usize {
        match self {
            Datatype::Word  => 4usize,
            Datatype::Half  => 4usize,
            Datatype::Byte  => 1usize,
            Datatype::Ascii => 1usize,
        }
    }
}




// Pre symbol resolution + Pre address assignment + Pre block creation

#[derive(Debug)]
pub enum KeyValue {
    Op(Box<dyn Extension>),
    Pseudo(Box<dyn Pseudo>),
    AssemblyDirective(Box<dyn Directive>),
    LinkerDirective(String),
    Section(SectionName),
    Label(String),
}




#[derive(Clone, Debug)]
pub enum ArgValue {
    Byte(u8),
    Number(i32),
    Register(Register),
    Offset(usize, i32),
    Literal(String),
    Use(String),
    UseHi(String),
    UseLo(String),
}

impl ArgValue {
    pub fn to_number(&self) -> Option<i32> {
        match self {
            ArgValue::Byte(b)            => Some((*b).try_into().unwrap()),
            ArgValue::Number(n)          => Some(*n),
            ArgValue::Register(register) => Some(register.id().into()),
            ArgValue::Offset(abs_addr, rel_addr)       => {
                todo!();
            },
            _ => None
        }
    }
}




pub struct OpcodeLine {
    pub(crate) keyword: Box<dyn Extension>,
    pub(crate) args: Vec<ArgValue>
}

impl Into<GenericLine> for OpcodeLine {
    fn into(self) -> GenericLine {
        GenericLine{
            keyword: KeyValue::Op(self.keyword),
            args: self.args
        }
    }
}



pub struct PseudoLine {
    pub(crate) keyword: Box<dyn Pseudo>,
    pub(crate) args: Vec<ArgValue>
}

impl Into<GenericLine> for PseudoLine {
    fn into(self) -> GenericLine {
        GenericLine{
            keyword: KeyValue::Pseudo(self.keyword),
            args: self.args
        }
    }
}




#[derive(Debug)]
pub struct GenericLine {
    pub(crate) keyword: KeyValue,
    pub(crate) args: Vec<ArgValue>
}

impl GenericLine {
    pub fn size_bytes_with_alignment(&self, alignment: usize) -> usize {
        match &self.keyword {
            KeyValue::Op(_) => 4usize,
            KeyValue::AssemblyDirective(_) => {
                let len = self.args.len();
                let exceeding = len % alignment;
                //ensure word alignment for sections
                let pad = if exceeding > 0 { alignment - exceeding } else { 0 };
                len + pad
            },
            _ => 0usize
        }
    }
}




#[derive(Debug)]
pub struct GenericBlock {
    pub(crate) name: SectionName,
    pub(crate) lines: Vec<GenericLine>
}
