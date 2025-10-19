pub enum MachineState {
    Exit(i32),
    Ok,
}

#[derive(Debug)]
pub enum MachineError {
    UnknownInstruction(u32),
    UnhandledInstruction(String),
}

pub trait Machine {
    // Init
    fn from_bytes_size(byte_count: usize, machine_endian: DataEndianness) -> Self ;
    fn from_words_size(word_count: usize, machine_endian: DataEndianness) -> Self ;
    fn from_bytes(data: &Vec<u8>, machine_endian: DataEndianness) -> Self ;
    fn from_words(data: &Vec<u32>, machine_endian: DataEndianness) -> Self ;


    // Core
    fn load(&mut self, start_addr: usize, instrs: &Vec<u32>) -> () ;
    fn fetch(&self) -> u32 ;
    fn jump(&mut self, off: usize) -> () ;
    fn set_pc(&mut self, new_pc: usize) -> () ;
    fn decode(&mut self) -> Result<MachineState, MachineError> ;
    fn endianness(&self) -> DataEndianness ;


    // CPU
    fn read_registers(&self) -> Vec<u32> ;
    fn write_registers(&mut self, gprs: Vec<u32>, pc: usize) -> () ;
    fn read_pc(&self) -> u32;


    // Memory
    fn bytes_count(&self) -> usize ;
    fn words_count(&self) -> usize ;

    fn bytes(&self) -> Vec<u8> ;
    fn words(&self) -> Vec<u32> ;

    fn read_memory_byte(&self, addr: usize) -> u8;
    fn write_memory_byte(&mut self, addr: usize, value: u8) -> () ;

    fn read_memory_bytes(&self, addr: usize, count: usize, alignment: usize) -> Vec<u8> ;
    fn write_memory_bytes(&mut self, addr: usize, values: &[u8]) -> () ;

    fn read_memory_word(&self, addr: usize) -> u32 ;
    fn write_memory_word(&mut self, addr: usize, value: u32) -> () ;

    fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> ;
    fn write_memory_words(&mut self, addr: usize, values: &[u32]) -> () ;


    // Debug
    fn assert_reg(&self, reg: u32, val: u32) -> bool ;
    fn assert_pc(&self, val: u32) -> bool ;
    fn assert_memory_words(&self, addr: usize, word_count: usize, values: &[u32]) -> bool ;
    fn assert_memory_bytes(&self, addr: usize, byte_count: usize, values: &[u8], alignment: usize) -> bool ;
    fn predict_next_pc(&self) -> usize ;
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
use crate::lang::ext::{
    InstructionFormat,
    Immediate,
};
use crate::lang::highassembly::Register;
use crate::lang::lowassembly::DataEndianness;
use crate::utils::set_remaining_bits;

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
        mem.write_bytes(0, data, machine_endian);
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

    fn load(&mut self, start_addr: usize, instrs: &Vec<u32>) -> () {
        self.mem.write_words(start_addr, instrs);
    }

    fn fetch(&self) -> u32  {
        let pc = self.cpu.read_pc();
        self.mem.read_word(pc)
    }

    fn jump(&mut self, off: usize) -> () {
        let pc = self.cpu.read_pc();
        self.cpu.write_pc(pc + off);
    }

    fn set_pc(&mut self, new_pc: usize) -> () {
        self.cpu.write_pc(new_pc);
    }

    fn decode(&mut self) -> Result<MachineState, MachineError> {
        let word = self.fetch();
        if let Some(ifmt) = InstructionFormat::decode(word) {
            let new_pc = predict_next_pc(self, &ifmt);
            // println!("{}: {:?} -> {}", self.read_pc(), &ifmt, new_pc);
            let state = handle(self, ifmt);
            if state.is_ok() {
                self.set_pc(new_pc);
            }
            state
        }
        else {
            eprintln!("WARNING: decode error: unknown word '{:x}'", word);
            Err(MachineError::UnknownInstruction(word))
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

    fn read_pc(&self) -> u32 {
        self.cpu.read_pc() as u32
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

    fn write_memory_bytes(&mut self, addr: usize, values: &[u8]) -> () {
        self.mem.write_bytes(addr, values, self.endian)
    }

    fn read_memory_word(&self, addr: usize) -> u32 {
        self.mem.read_word(addr)
    }

    fn write_memory_word(&mut self, addr: usize, value: u32) -> () {
        self.mem.write_word(addr, value);
    }

    fn read_memory_words(&self, addr: usize, count: usize) -> Vec<u32> {
        self.mem.read_words(addr, count, self.endian)
    }

    fn write_memory_words(&mut self, addr: usize, values: &[u32]) -> () {
        self.mem.write_words(addr, values);
    }

    fn assert_reg(&self, reg: u32, val: u32) -> bool {
        self.cpu.read(reg as usize) == val
    }

    fn assert_pc(&self, val: u32) -> bool {
        (self.cpu.read_pc() as u32) == val
    }

    fn assert_memory_words(&self, addr: usize, word_count: usize, values: &[u32]) -> bool {
        let words = self.mem.read_words(addr, word_count, self.endian);
        for (word_in_memory, word_test) in words.iter().zip(values) {
            if word_in_memory != word_test {
                return false;
            }
        }
        true
    }

    fn assert_memory_bytes(&self, addr: usize, byte_count: usize, values: &[u8], alignment: usize) -> bool {
        let bytes = self.mem.read_bytes(addr, byte_count, self.endian, alignment);
        for (byte_in_memory, byte_test) in bytes.iter().zip(values) {
            if byte_in_memory != byte_test {
                return false;
            }
        }
        true
    }

    fn predict_next_pc(&self) -> usize {
        let word = self.fetch();
        if let Some(ifmt) = InstructionFormat::decode(word) {
            predict_next_pc(self, &ifmt)
        }
        else {
            self.cpu.read_pc()
        }
    }
}

fn handle(m: &mut SimpleMachine, ifmt: InstructionFormat) -> Result<MachineState, MachineError> {
    match ifmt {
        InstructionFormat::R { funct7, rs2, rs1, funct3, rd, opcode: _ } => {
            let v1 = m.cpu.read(rs1 as usize);
            let v2 = m.cpu.read(rs2 as usize);
            let res = match (funct7, funct3) {
                (0b0000000, 0b000) => { v1 + v2  }, //ADD
                (0b0100000, 0b000) => { ((v1 as i32) - (v2 as i32)) as u32 }, //SUB
                (0b0000000, 0b111) => { v1 & v2  }, //AND
                (0b0000000, 0b110) => { v1 | v2  }, //OR
                (0b0000000, 0b100) => { v1 ^ v2  }, //XOR
                (0b0000000, 0b001) => { v1 << v2 }, //SLL
                (0b0000000, 0b101) => { v1 >> v2 }, //SRL
                (0b0000001, 0b000) => { //MUL
                    let v1: i64 = v1.into();
                    let v2: i64 = v2.into();
                    (v1 * v2) as u32
                },
                (0b0000001, 0b001) => { //MULH
                    let v1: i64 = v1.into();
                    let v2: i64 = v2.into();
                    ((v1 * v2) >> 32) as u32
                },
                (0b0000001, 0b010) => { //MULHSU
                    let sign: i32 = ((v1 as i32) > 0).into();
                    let v1: u64 = v1.into();
                    let v2: u64 = v2.into();
                    (sign * (((v1 * v2) >> 32) as i32) ) as u32
                },
                (0b0000001, 0b011) => { //MULHU
                    let v1: u64 = v1.into();
                    let v2: u64 = v2.into();
                    ((v1 * v2) >> 32) as u32
                },
                (0b0000001, 0b100) => { ((v1 as i32) / (v2 as i32)) as u32 }, //DIV
                (0b0000001, 0b101) => { v1 / v2 }, //DIVU
                (0b0000001, 0b110) => { ((v1 as i32) % (v2 as i32)) as u32 }, //REM
                (0b0000001, 0b111) => { v1 % v2 }, //REMU
                _ => {
                    let errmsg = format!("Unhandled R: (f7, f3) = ({}, {})", funct7, funct3);
                    return Err(MachineError::UnhandledInstruction(errmsg));
                }
            };
            m.cpu.write(rd as usize, res);
        },
        InstructionFormat::I { imm, rs1, funct3, rd, opcode } => {
            let rs1_val = m.cpu.read(rs1 as usize);
            let imm = imm.decode();
            let opt = match (funct3, opcode) {
                (0b000, 0b0010011) => { Some(rs1_val.wrapping_add_signed(imm as i32)) }, // ADDI
                (0b111, 0b0010011) => { Some(rs1_val & imm) }, // ANDI
                (0b110, 0b0010011) => { Some(rs1_val | imm) }, // ORI
                (0b100, 0b0010011) => { Some(rs1_val ^ imm) }, // XORI
                (0b000, 0b1100111) => { Some(m.cpu.read_pc() as u32 + 4) }, // JALR
                (0b010, 0b0000011) => {
                    let addr = rs1_val.saturating_add_signed(imm as i32) as usize;
                    Some(m.read_memory_word(addr))
                }, // LW
                (0b000, 0b0000011) => {
                    let addr = rs1_val.saturating_add_signed(imm as i32) as usize;
                    Some(m.read_memory_byte(addr) as u32)
                }, // LB
                (0b000, 0b1110011) => {
                    let a7 = m.cpu.read(Register::A7.id().into()) as usize;
                    if let Some(sys) = Sysno::new(a7) {
                        match sys {
                            Sysno::write => {
                            },
                            Sysno::exit => {
                                let a0 = m.cpu.read(Register::A0.id().into()) as usize;
                                return Ok(MachineState::Exit(a0 as i32));
                            },
                            _ => {
                            }
                        }
                    }
                    None
                }, // ECALL
                (0b100, 0b0000011) => None, // LBU
                _ => {
                    let errmsg = format!("Unhandled I: (f3, op) = ({}, {})", funct3, opcode);
                    return Err(MachineError::UnhandledInstruction(errmsg));
                }
            };
            if let Some(res) = opt {
                m.cpu.write(rd as usize, res as u32);
            }
        },
        InstructionFormat::S { imm, rs2, rs1, funct3, opcode } => {
            let rs1 = m.cpu.read(rs1 as usize);
            let rs2 = m.cpu.read(rs2 as usize);
            let imm = imm.decode();
            let addr = (rs1 + imm) as usize;
            let val  = rs2;
            match (funct3, opcode) {
                (0b010, 0b0100011) => m.mem.write_word(addr, val), //SW
                (0b000, 0b0100011) => m.mem.write_byte(addr, (val & 0b1111_1111) as u8), //SB
                _ => {
                    let errmsg = format!("Unhandled S: (f3, op) = ({}, {})", funct3, opcode);
                    return Err(MachineError::UnhandledInstruction(errmsg));
                }
            }
        },
        InstructionFormat::U { imm, rd, opcode } => {
            let upper20bits = imm.decode();
            match opcode {
                0b0110111 => m.cpu.write(rd as usize, upper20bits), //LUI
                0b0010111 => { //AUIPC
                    let pc   = m.cpu.read_pc();
                    let addr = pc.saturating_add_signed(upper20bits as isize);
                    m.cpu.write(rd as usize, addr as u32);
                },
                _ => {
                    let errmsg = format!("Unhandled U: op = {}", opcode);
                    return Err(MachineError::UnhandledInstruction(errmsg));
                }
            }
        },
        InstructionFormat::J { imm: _, rd, opcode } => {
            match opcode {
                0b1101111  => {
                    let pc = m.cpu.read_pc();
                    let ret_addr = pc + 4;
                    m.cpu.write(rd as usize, ret_addr as u32);
                }, //JAL
                _ => {
                    let errmsg = format!("Unhandled J: op = {}", opcode);
                    return Err(MachineError::UnhandledInstruction(errmsg));
                }
            }

        },
        InstructionFormat::B { .. } => {
        },
    }

    Ok(MachineState::Ok)
}


fn predict_next_pc(m: &SimpleMachine, ifmt: &InstructionFormat) -> usize {
    let pc = m.read_pc();
    match ifmt {
        // JALR
        InstructionFormat::I { imm, rs1, funct3: 0b000, rd: _, opcode: 0b1100111 } => {
            let rs1_val = m.cpu.read(*rs1 as usize);
            let imm = imm.decode();
            let rel_addr = rs1_val.saturating_add_signed(imm as i32);
            return rel_addr as usize;
        },
        InstructionFormat::B { imm, rs2, rs1, funct3, opcode } => {
            let rs1 = m.cpu.read(*rs1 as usize);
            let rs2 = m.cpu.read(*rs2 as usize);
            let cond = match (funct3, opcode) {
                (0b000, 0b1100011) => rs1 == rs2, //BEQ
                (0b001, 0b1100011) => rs1 != rs2, //BNE
                (0b100, 0b1100011) => rs1 < rs2,  //BLT
                (0b101, 0b1100011) => rs1 >= rs2, //BGE
                _ => panic!("Unhandled B: (f3, op) = ({}, {})", funct3, opcode),
            };
            if cond {
                let imm = imm.decode();
                let beq_addr = m.cpu.read_pc() as u32;
                let rel_addr = beq_addr.saturating_add_signed(imm as i32);
                return rel_addr as usize;
            }
            else {
                return (pc as usize) + 4usize;
            }
        },
        // JAL
        InstructionFormat::J { imm, rd: _, opcode: 0b1101111 } => {
            let imm = imm.decode();
            let next_pc = pc.saturating_add_signed(imm as i32);
            return next_pc as usize;
        },
        _ => return (pc as usize) + 4usize,
    }
}
