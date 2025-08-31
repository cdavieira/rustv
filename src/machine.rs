// TODO: create test to all these methods

pub trait Machine {
    // Core
    fn new(data: &Vec<u32>) -> Self ;
    fn load(&mut self, instrs: &Vec<u32>) -> () ;
    fn fetch(&self) -> u32 ;
    fn jump(&mut self, off: usize) -> () ;
    fn decode(&mut self) -> () ;


    // CPU
    // This
    fn read_registers(&self) -> Vec<u32> ;
    // This
    fn write_registers(&mut self, gprs: Vec<u32>, pc: usize) -> () ;


    // Memory
    fn bytes_count(&self) -> usize ;
    fn words_count(&self) -> usize ;

    fn bytes(&self) -> Vec<u8> ;
    fn words(&self) -> Vec<u32> ;

    fn read_memory_byte(&self, addr: usize) -> u8;
    // This
    fn write_memory_byte(&mut self, addr: usize, value: u8) -> () ;

    // This
    fn read_memory_bytes(&self, addr: usize, count: usize) -> Vec<u8> ;
    fn write_memory_bytes(&mut self, addr: usize, values: &Vec<u8>) -> () ;

    fn read_memory_word(&self, addr: usize) -> u32 ;
    fn write_memory_word(&mut self, addr: usize, value: u32) -> () ;

    fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> ;
    fn write_memory_words(&mut self, addr: usize, values: &Vec<u32>) -> () ;


    // Debug
    fn assert_reg(&self, reg: u32, val: u32) -> bool ;
    fn assert_memory_words(&self, addr: usize, word_count: usize, values: &Vec<u32>) -> bool ;
    fn assert_memory_bytes(&self, addr: usize, byte_count: usize, values: &Vec<u8>) -> bool ;
}



/* Possible implementation */

use crate::cpu::{SimpleCPU, CPU};
use crate::memory::{SimpleMemory, Memory};
use crate::spec::InstructionFormat;

pub struct SimpleMachine {
    cpu: SimpleCPU,
    mem: SimpleMemory
}

impl Machine for SimpleMachine {
    fn new(data: &Vec<u32>) -> Self {
        let mut mem = SimpleMemory::new();
        mem.reserve_words(data.len());
        mem.write_words(0, data);
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
        }
    }

    fn load(&mut self, instrs: &Vec<u32>) -> () {
        self.mem.write_words(0, instrs);
    }

    fn fetch(&self) -> u32  {
        let pc = self.cpu.read_pc();
        self.mem.read_word(pc)
    }

    fn jump(&mut self, off: usize) -> () {
        let pc = self.cpu.read_pc();
        self.cpu.write_pc(pc + off);
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

    fn read_registers(&self) -> Vec<u32> {
        self.cpu.read_all()
    }

    fn write_registers(&mut self, gprs: Vec<u32>, pc: usize) -> () {
        self.cpu.write_all(gprs, pc);
    }

    fn bytes_count(&self) -> usize {
        self.mem.bytes_count()
    }

    fn words_count(&self) -> usize {
        self.mem.words_count()
    }

    fn bytes(&self) -> Vec<u8>  {
        self.mem.bytes()
    }

    fn words(&self) -> Vec<u32> {
        self.mem.words()
    }

    fn read_memory_byte(&self, addr: usize) -> u8 {
        self.mem.read_byte(addr)
    }

    fn write_memory_byte(&mut self, addr: usize, value: u8) -> () {
        self.mem.write_byte(addr, value)
    }

    fn read_memory_bytes(&self, addr: usize, count: usize) -> Vec<u8> {
        self.mem.read_bytes(addr, count)
    }

    fn write_memory_bytes(&mut self, addr: usize, values: &Vec<u8>) -> () {
        self.mem.write_bytes(addr, values)
    }

    fn read_memory_word(&self, addr: usize) -> u32 {
        self.mem.read_word(addr)
    }

    fn write_memory_word(&mut self, addr: usize, value: u32) -> () {
        self.mem.write_word(addr, value);
    }

    fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> {
        self.mem.read_words(addr, count)
    }

    fn write_memory_words(&mut self, addr: usize, values: &Vec<u32>) -> () {
        self.mem.write_words(addr, values);
    }

    fn assert_reg(&self, reg: u32, val: u32) -> bool {
        self.cpu.read(reg as usize) == val
    }

    fn assert_memory_words(&self, addr: usize, word_count: usize, values: &Vec<u32>) -> bool {
        let words = self.mem.read_words(addr, word_count);
        for (word_in_memory, word_test) in words.iter().zip(values) {
            if word_in_memory != word_test {
                return false;
            }
        }
        true
    }

    fn assert_memory_bytes(&self, addr: usize, byte_count: usize, values: &Vec<u8>) -> bool {
        let bytes = self.mem.read_bytes(addr, byte_count);
        for (byte_in_memory, byte_test) in bytes.iter().zip(values) {
            if byte_in_memory != byte_test {
                return false;
            }
        }
        true
    }
}
