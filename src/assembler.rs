pub trait Assembler {
    type Instruction;
    fn to_words(&self, instructions: Vec<Self::Instruction>) -> Vec<u32> ;
}

/**/

use crate::spec::{instruction_to_binary, AssemblyInstruction, KeyValue};

pub fn to_u32(instructions: Vec<AssemblyInstruction>) -> Vec<u32> {
    let mut insts = Vec::new();
    for inst in instructions {
        match inst.key {
            KeyValue::OP(extension) => {
                insts.push((inst.addr, extension, inst.args))
            },
            _ => {}
        }
    }
    insts
        .into_iter()
        .map(|i| instruction_to_binary(&i.1, &i.2))
        .collect()
}
