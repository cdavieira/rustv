pub trait Assembler {
    type Instruction;
    fn to_words(&self, instructions: Vec<Self::Instruction>) -> Vec<u32> ;
}

/**/

use crate::spec::{Extension, ArgKey, ArgValue, instruction_to_binary};

pub fn to_u32(instructions: Vec<(Box<dyn Extension>, Vec<ArgValue>)>) -> Vec<u32> {
    let mut v = Vec::new();
    for inst in instructions {
        v.push(instruction_to_binary(&inst.0, &inst.1));
    }
    v
}
