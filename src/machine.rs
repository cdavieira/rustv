pub trait Machine {
    fn fetch(&self) -> u32 ;
    fn jump(&mut self, off: usize) -> () ;

    fn load(&mut self, instrs: &Vec<u32>) -> () ;
    fn bytes(&self) -> Vec<u8> ;
    fn words(&self) -> Vec<u32> ;

    fn decode(&mut self) -> () ;

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

// TODO: move all these methods to the Machine interface

// TODO: create test to all these methods

impl SimpleMachine {
    pub fn new(data: &Vec<u32>) -> Self {
        let mut mem = BasicMemory::new();
        mem.reserve_words(data.len());
        mem.write_words(0, data);
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
        }
    }

    // This
    pub fn read_registers(&self) -> Vec<u32> {
        self.cpu.read_all()
    }

    // This
    pub fn write_registers(&mut self, gprs: Vec<u32>, pc: usize) -> () {
        self.cpu.write_all(gprs, pc);
    }

    pub fn read_memory_byte(&self, addr: usize) -> u8 {
        self.mem.read_byte(addr)
    }

    // This
    pub fn write_memory_byte(&mut self, addr: usize, value: u8) -> () {
        self.mem.write_byte(addr, value)
    }

    // This
    pub fn read_memory_bytes(&self, addr: usize, count: usize) -> Vec<u8> {
        self.mem.read_bytes(addr, count)
    }

    pub fn write_memory_bytes(&mut self, addr: usize, values: &Vec<u8>) -> () {
        self.mem.write_bytes(addr, values)
    }

    pub fn read_memory_word(&self, addr: usize) -> u32 {
        self.mem.read_word(addr)
    }

    pub fn write_memory_word(&mut self, addr: usize, value: u32) -> () {
        self.mem.write_word(addr, value);
    }

    pub fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> {
        self.mem.read_words(addr, count)
    }

    pub fn write_memory_words(&mut self, addr: usize, values: &Vec<u32>) -> () {
        self.mem.write_words(addr, values);
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

    fn load(&mut self, instrs: &Vec<u32>) -> () {
        self.mem.write_words(0, instrs);
    }

    fn decode(&mut self) -> ()  {
        let word = self.fetch();
        self.jump(4usize);

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
        self.mem.words()
    }

    fn assert_reg(&self, reg: u32, val: u32) -> bool {
        self.cpu.read(reg as usize) == val
    }
}
