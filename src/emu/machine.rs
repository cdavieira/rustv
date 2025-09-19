// TODO: create test to all these methods

pub trait Machine {
    // Init
    fn from_bytes_size(byte_count: usize, machine_endian: DataEndianness) -> Self ;
    fn from_words_size(word_count: usize, machine_endian: DataEndianness) -> Self ;
    fn from_bytes(data: &Vec<u8>, machine_endian: DataEndianness) -> Self ;
    fn from_words(data: &Vec<u32>, machine_endian: DataEndianness) -> Self ;


    // Core
    fn load(&mut self, instrs: &Vec<u32>) -> () ;
    fn fetch(&self) -> u32 ;
    fn jump(&mut self, off: usize) -> () ;
    fn decode(&mut self) -> () ;
    fn endianness(&self) -> DataEndianness ;


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
    fn read_memory_bytes(&self, addr: usize, count: usize, alignment: usize) -> Vec<u8> ;
    fn write_memory_bytes(&mut self, addr: usize, values: &Vec<u8>) -> () ;

    fn read_memory_word(&self, addr: usize) -> u32 ;
    fn write_memory_word(&mut self, addr: usize, value: u32) -> () ;

    fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> ;
    fn write_memory_words(&mut self, addr: usize, values: &Vec<u32>) -> () ;


    // Debug
    fn assert_reg(&self, reg: u32, val: u32) -> bool ;
    fn assert_memory_words(&self, addr: usize, word_count: usize, values: &Vec<u32>) -> bool ;
    fn assert_memory_bytes(&self, addr: usize, byte_count: usize, values: &Vec<u8>, alignment: usize) -> bool ;
}



/* Possible implementation */

use syscalls::riscv32::Sysno;
use crate::emu::{
    cpu::SimpleCPU,
    cpu::CPU,
};
use crate::emu::{
    memory::SimpleMemory,
    memory::Memory
};
use crate::lang::ext::{Extension, InstructionFormat, RV32I};
use crate::lang::highassembly::Register;
use crate::utils::DataEndianness;

pub struct SimpleMachine {
    cpu: SimpleCPU,
    mem: SimpleMemory,
    endian: DataEndianness,
}

impl Machine for SimpleMachine {
    fn from_bytes_size(byte_count: usize, machine_endian: DataEndianness) -> Self  {
        let mut mem = SimpleMemory::new(DataEndianness::Be);
        mem.reserve_bytes(byte_count);
        for i in 0..byte_count {
            mem.write_byte(i, 0);
        }
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
            endian: machine_endian
        }
    }

    fn from_words_size(word_count: usize, machine_endian: DataEndianness) -> Self  {
        let mut mem = SimpleMemory::new(DataEndianness::Be);
        mem.reserve_words(word_count);
        for i in 0..word_count {
            mem.write_word(i, 0);
        }
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
            endian: machine_endian
        }
    }

    fn from_bytes(data: &Vec<u8>, machine_endian: DataEndianness) -> Self  {
        let mut mem = SimpleMemory::new(DataEndianness::Be);
        mem.reserve_bytes(data.len());
        mem.write_bytes(0, data);
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
            endian: machine_endian
        }
    }

    fn from_words(data: &Vec<u32>, machine_endian: DataEndianness) -> Self {
        let mut mem = SimpleMemory::new(DataEndianness::Be);
        mem.reserve_words(data.len());
        mem.write_words(0, data);
        SimpleMachine {
            cpu: SimpleCPU::new(),
            mem,
            endian: machine_endian
        }
    }

    fn load(&mut self, instrs: &Vec<u32>) -> () {
        self.mem.write_words(0, instrs);
    }

    fn fetch(&self) -> u32  {
        let pc = self.cpu.read_pc();
        self.mem.read_word(pc, self.endian)
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
                let v1 = self.cpu.read(rs1 as usize);
                let v2 = self.cpu.read(rs2 as usize);
                match (funct7, funct3) {
                    (0b0000000, 0b000) => {
                        self.cpu.write(rd as usize, v1 + v2);
                    }, //ADD
                    (0b0100000, 0b000) => {
                        self.cpu.write(rd as usize, v1 - v2);
                    }, //SUB
                    (0b0000000, 0b111) => {
                        self.cpu.write(rd as usize, v1 & v2);
                    }, //AND   
                    (0b0000000, 0b110) => {
                        self.cpu.write(rd as usize, v1 | v2);
                    }, //OR    
                    (0b0000000, 0b100) => {
                        self.cpu.write(rd as usize, v1 ^ v2);
                    }, //XOR   
                    (0b0000000, 0b001) => {
                        self.cpu.write(rd as usize, v1 << v2);
                    }, //SLL   
                    (0b0000000, 0b101) => {
                        self.cpu.write(rd as usize, v1 >> v2);
                    }, //SRL   
                    _ => {
                        panic!("Unhandled R: (f7, f3) = ({}, {})", funct7, funct3);
                    }
                }
            },
            InstructionFormat::I { imm, rs1, funct3, rd, opcode } => {
                let vs = self.cpu.read(rs1 as usize);
                match (funct3, opcode) {
                    (0b000, 0b1110011) => {
                        let a7: usize = self.cpu
                            .read(Register::A7.id().into())
                            .try_into()
                            .unwrap();
                        if let Some(sys) = Sysno::new(a7) {
                            match sys {
                                Sysno::write => {
                                },
                                Sysno::exit => {
                                },
                                _ => {
                                }
                            }
                        }
                    }, // ECALL 
                    (0b000, 0b1100111) => {
                        todo!();
                        // let ret_addr = self.cpu.read_pc() + 4;
                        // let v = vs | imm;
                        // self.cpu.write(rd as usize, v);
                    }, // JALR  
                    (0b000, 0b0010011) => {
                        let v = vs + imm;
                        self.cpu.write(rd as usize, v);
                    }, // ADDI  
                    (0b111, 0b0010011) => {
                        let v = vs & imm;
                        self.cpu.write(rd as usize, v);
                    }, // ANDI  
                    (0b110, 0b0010011) => {
                        let v = vs | imm;
                        self.cpu.write(rd as usize, v);
                    }, // ORI   
                    (0b100, 0b0010011) => {
                        let v = vs ^ imm;
                        self.cpu.write(rd as usize, v);
                    }, // XORI  
                    (0b010, 0b0000011) => {
                        // TODO: use sign extension
                        let addr = vs + (imm << 20);
                        let v = self.mem.read_word(addr as usize, self.endian);
                        self.cpu.write(rd as usize, v);
                    }, // LW
                    (0b000, 0b0000011) => {
                        let addr = vs + (imm << 20);
                        let v = self.mem.read_byte(addr as usize);
                        self.cpu.write(rd as usize, v as u32);
                    }, // LB
                    (0b100, 0b0000011) => {
                        todo!();
                    }, // LBU
                    _ => {
                        panic!("Unhandled I: (f3, op) = ({}, {})", funct3, opcode);
                    }
                }
            },
            InstructionFormat::S { imm1, rs2, rs1, funct3, imm2, opcode } => {
                match (funct3, opcode) {
                    (0b010, 0b0100011 ) => { //SW
                        let v = self.cpu.read(rs2 as usize);
                        let imm = (imm1 << 5) & imm2;
                        let addr = self.cpu.read(rs1 as usize) + imm;
                        self.mem.write_word(addr as usize, v);
                    },
                    (0b000, 0b0100011 ) => { //SB
                        let v = (self.cpu.read(rs2 as usize)) & 0b1111_1111;
                        let imm = (imm1 << 5) & imm2;
                        let addr = self.cpu.read(rs1 as usize) + imm;
                        self.mem.write_byte(addr as usize, v as u8);
                    },
                    _ => {
                        panic!("Unhandled S: (f3, op) = ({}, {})", funct3, opcode);
                    }
                }
            },
            // TODO: shouldn't imm be an i32?
            InstructionFormat::B { imm1, rs2, rs1, funct3, imm2, opcode } => {
                match (funct3, opcode) {
                    (0b0, 0b1100011) => { //BEQ
                        if self.cpu.read(rs1 as usize) == self.cpu.read(rs2 as usize) {
                            let imm = (imm1 << 5) & imm2;
                            let pc = self.cpu.read_pc() as u32;
                            let addr = pc + imm;
                            self.cpu.write_pc(addr as usize);
                        }
                    },
                    _ => {
                        panic!("Unhandled B: (f3, op) = ({}, {})", funct3, opcode);
                    }
                }
            },
            InstructionFormat::U { imm, rd, opcode } => {
                match opcode {
                    0b0110111 => { //LUI
                        let upper20bits = imm << 12;
                        self.cpu.write(rd as usize, upper20bits);
                    },
                    0b0010111 => { //AUIPC
                        let offset = (imm << 12) as i32;
                        let pc: u32 = self.cpu.read_pc().try_into().unwrap();
                        let addr = (pc as i32) + offset;
                        self.cpu.write(rd as usize, addr as u32);
                    },
                    _ => {
                        panic!("Unhandled U: op = {}", opcode);
                    }
                }

            },
            InstructionFormat::J { imm: _, rd: _, opcode } => {
                match opcode {
                    0b1101111  => { //JAL
                        todo!();
                    },
                    _ => {
                        panic!("Unhandled J: op = {}", opcode);
                    }
                }

            },
        }
    }

    fn endianness(&self) -> DataEndianness  {
        self.endian
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

    fn read_memory_bytes(&self, addr: usize, count: usize, alignment: usize) -> Vec<u8> {
        self.mem.read_bytes(addr, count, self.endian, alignment)
    }

    fn write_memory_bytes(&mut self, addr: usize, values: &Vec<u8>) -> () {
        self.mem.write_bytes(addr, values)
    }

    fn read_memory_word(&self, addr: usize) -> u32 {
        self.mem.read_word(addr, self.endian)
    }

    fn write_memory_word(&mut self, addr: usize, value: u32) -> () {
        self.mem.write_word(addr, value);
    }

    fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> {
        self.mem.read_words(addr, count, self.endian)
    }

    fn write_memory_words(&mut self, addr: usize, values: &Vec<u32>) -> () {
        self.mem.write_words(addr, values);
    }

    fn assert_reg(&self, reg: u32, val: u32) -> bool {
        self.cpu.read(reg as usize) == val
    }

    fn assert_memory_words(&self, addr: usize, word_count: usize, values: &Vec<u32>) -> bool {
        let words = self.mem.read_words(addr, word_count, self.endian);
        for (word_in_memory, word_test) in words.iter().zip(values) {
            if word_in_memory != word_test {
                return false;
            }
        }
        true
    }

    fn assert_memory_bytes(&self, addr: usize, byte_count: usize, values: &Vec<u8>, alignment: usize) -> bool {
        let bytes = self.mem.read_bytes(addr, byte_count, self.endian, alignment);
        for (byte_in_memory, byte_test) in bytes.iter().zip(values) {
            if byte_in_memory != byte_test {
                return false;
            }
        }
        true
    }
}
