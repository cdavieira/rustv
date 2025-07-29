pub trait Assembler {
    type Instruction;
    fn to_words(&self, instructions: Vec<Self::Instruction>) -> Vec<u32> ;
}

/**/

use crate::{lexer, spec::{instruction_to_binary, ArgKey, ArgValue, Extension}};

pub fn to_u32(instructions: Vec<(usize, lexer::Token, Vec<ArgValue>)>) -> Vec<u32> {
    let mut insts = Vec::new();
    for inst in instructions {
        match inst.1 {
            lexer::Token::OP(extension) => {
                insts.push((inst.0, extension, inst.2))
            },
            _ => {}
        }
    }
    insts
        .into_iter()
        .map(|i| instruction_to_binary(&i.1, &i.2))
        .collect()
}
