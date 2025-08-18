use crate::spec::AssemblyData;

pub trait Assembler {
    type Input;
    fn to_words(&self, instructions: Self::Input) -> AssemblyData ;
}

/**/

use crate::spec::{instruction_to_binary, AssemblySection, KeyValue};

pub fn to_u32(section: AssemblySection) -> AssemblyData {
    let mut data = Vec::new();

    for i in &section.instructions {
        // println!("Processing {:?}", &i.key);
        match &i.key {
            KeyValue::OP(extension) => {
                let word = instruction_to_binary(extension, &i.args);
                // println!("Turned into {}", word);
                data.push(word);
            },
            KeyValue::DIRECTIVE(_) => {
                let words: Vec<u32> = i.args
                        .iter()
                        .map(|x| *x as u32)
                        .collect();
                // println!("Turned into {:?}", words);
                data.extend(words);
            },
            _ => {}
        }
    }
    // println!("Len 1: {}", data.len());

    let (addr, name) = (section.addr, section.name);
    AssemblyData {
        addr,
        name,
        data,
    }
}
