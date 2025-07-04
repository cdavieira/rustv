use crate::memory::{BasicMemory, Memory};
use crate::cpu::{SimpleCPU, CPU};


pub trait Machine {
    fn fetch(&self) -> u32 ;

    //TODO: should i execute a instruction as a u32 or as a Statement?
    fn decode(&mut self, instr: u32) -> () ;

    fn cycle(&mut self) -> () ;
}



/* Possible implementation */

pub struct BasicMachine {
    pub memory: BasicMemory,
    pub cpu: SimpleCPU,
}

impl BasicMachine {
    pub fn new() -> Self {
        BasicMachine{
            memory: BasicMemory::new(),
            cpu: SimpleCPU::new(),
        }
    }

    pub fn info(&self) -> () {
        self.cpu.info();
    }
}

impl Machine for BasicMachine {
    fn fetch(&self) -> u32 {
        // self.memory.read_word(self.cpu.pc)
        todo!();
    }

    fn decode(&mut self, instr: u32) -> () {
        todo!();
    }

    fn cycle(&mut self) -> () {
        let inst = self.fetch();
        self.decode(inst);
    }
}
