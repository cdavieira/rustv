use crate::lang::highassembly::ArgValue;

use super::highassembly::Datatype;




/**
A directive was thought to be a sequence of tokens which can be turned into a sequence of
raw bytes (4-byte aligned)

The length of the resulting vector is expected to be a multiple of 4 (to ensure 4bytes alignment)
*/
pub trait Directive: std::fmt::Debug {
    fn translate(&self, args: &Vec<ArgValue>) -> Vec<u8> ;
    fn datatype(&self) -> Datatype ;
}





// Assembly Directives implementation

#[derive(Debug)]
pub enum DirectiveInstruction {
    Word,
    Half,
    Byte,
    Skip,
    Ascii,
}

// WARNING: when translating a directive into its sequence of bytes, the resulting endianness
// should be little endian, as to standardize how this data gets handled later on. If this doesn't
// happen, then things might not work
// TODO: handle more than 1 byte/word/ascii?
impl Directive for DirectiveInstruction {
    fn translate(&self, args: &Vec<ArgValue>) -> Vec<u8>  {
        match self {
            DirectiveInstruction::Byte => {
                args.iter()
                    .map(|arg| {
                        match arg {
                            ArgValue::Number(n) => *n as u8,
                            _ => panic!("Byte directive got something other than a number"),
                        }
                    })
                    .collect()
            },
            DirectiveInstruction::Word => {
                args.iter()
                    .map(|arg| {
                        match arg {
                            ArgValue::Number(n) => n.to_le_bytes().to_vec(),
                            _ => panic!("WORD directive got something other than a number"),
                        }
                    })
                    .flatten()
                    .collect()
            },
            DirectiveInstruction::Ascii => {
                match &args[0] {
                    ArgValue::Literal(s) => s.bytes().collect(),
                    _ => panic!("ASCII directive got something other than a literal"),
                }
            },
            DirectiveInstruction::Skip => {
                match &args[0] {
                    ArgValue::Number(n) => {
                        let capacity: usize = (*n).try_into().unwrap();
                        let mut v = Vec::new();
                        v.reserve(capacity + capacity % 4);
                        for _ in 0..*n {
                            v.push(0);
                        }
                        v
                    },
                    _ => panic!("SKIP directive got something other than a number"),
                }
            },
            _ => panic!("Unsupported directive")
        }
    }

    fn datatype(&self) -> Datatype {
        match self {
            DirectiveInstruction::Word  => Datatype::Word,
            DirectiveInstruction::Half  => Datatype::Half,
            DirectiveInstruction::Byte  => Datatype::Byte,
            DirectiveInstruction::Skip  => Datatype::Byte,
            DirectiveInstruction::Ascii => Datatype::Ascii,
        }
    }
}
