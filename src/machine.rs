pub trait Machine {
    fn fetch(&self) -> u32 ;
    fn jump(&mut self, off: usize) -> () ;

    fn load(&mut self, instrs: &Vec<u32>) -> () ;
    fn bytes(&self) -> Vec<u8> ;
    fn words(&self) -> Vec<u32> ;

    fn decode(&mut self) -> () ;

    fn info(&self) -> () ;

    fn assert_reg(&self, reg: u32, val: u32) -> bool ;
}

/* Possible implementation */

use crate::cpu::{SimpleCPU, CPU};
use crate::memory::{BasicMemory, Memory};
use crate::spec::InstructionFormat;

pub struct SimpleMachine {
    cpu: SimpleCPU,
    mem: BasicMemory
}

impl SimpleMachine {
    pub fn new(data: &Vec<u32>) -> Self {
        let mut mem = BasicMemory::new();
        mem.alloc(data.len() * 4);
        mem.load_words(data);
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
        }
    }
}

impl Machine for SimpleMachine {
    fn fetch(&self) -> u32  {
        let pc = self.cpu.read_pc();
        self.mem.read_word(pc)
    }

    fn jump(&mut self, off: usize) -> () {
        let pc = self.cpu.read_pc();
        self.cpu.write_pc(pc + off);
    }

    fn info(&self) -> () {
        self.cpu.info();
    }

    fn load(&mut self, instrs: &Vec<u32>) -> () {
        self.mem.load_words(instrs);
    }

    fn decode(&mut self) -> ()  {
        let word = self.fetch();
        self.jump(1usize);

        match InstructionFormat::decode(word) {
            InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode: _ } => {
                match (funct7, funct3) {
                    (0b0, 0b0) => { //ADD
                        let v1 = self.cpu.read(rs1 as usize);
                        let v2 = self.cpu.read(rs2 as usize);
                        self.cpu.write(rd as usize, v1 + v2);
                    },
                    _ => {

                    }
                }
            },
            InstructionFormat::I { imm, rs1, funct3, rd, opcode } => {
                match (funct3, opcode) {
                    (0b010, 0b0000011) => { //LW
                        // let r1 = self.cpu.read(rs1 as usize);
                        // let v1 = self.mem.read_word((r1 + imm))
                        // self.cpu.write(rd, v1);
                    },
                    (0b000, 0b0010011) => { //ADDI
                        let r1 = self.cpu.read(rs1 as usize);
                        let v = r1 + imm;
                        self.cpu.write(rd as usize, v);
                    }
                    _ => {

                    }
                }
            },
            InstructionFormat::S { imm1: _, rs2: _, rs1: _, funct3, imm2: _, opcode } => {
                match (funct3, opcode) {
                    (0b010, 0b0100011 ) => { //SW

                    },
                    _ => {

                    }
                }
            },
            InstructionFormat::B { imm1: _, rs2: _, rs1: _, funct3, imm2: _, opcode } => {
                match (funct3, opcode) {
                    (0b0, 0b1100011) => { //BEQ

                    },
                    _ => {

                    }
                }
            },
            InstructionFormat::U { imm: _, rd: _, opcode } => {
                match opcode {
                    0b0110111  => { //LUI
                    },
                    _ => {

                    }
                }

            },
            InstructionFormat::J { imm: _, rd: _, opcode } => {
                match opcode {
                    0b1101111  => { //JAL
                    },
                    _ => {

                    }
                }

            },
        }
    }

    fn bytes(&self) -> Vec<u8>  {
        self.mem.bytes()
    }

    fn words(&self) -> Vec<u32> {
        self.bytes()
            .chunks(4)
            .map(|chunk| {
                let b3: u32 = chunk[0].into();
                let b2: u32 = chunk[1].into();
                let b1: u32 = chunk[2].into();
                let b0: u32 = chunk[3].into();
                (b3 << 24) | (b2 << 16) | (b1 << 8) | b0
            })
            .collect()
    }

    fn assert_reg(&self, reg: u32, val: u32) -> bool {
        self.cpu.read(reg as usize) == val
    }
}
