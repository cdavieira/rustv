use rustv::syntax::intel;

//idea: convert a statement into its binary representation as specified in the riscv32 ISA
trait BinaryEncoder {
    type Statement;

    fn to_binary(&self, stat: Self::Statement) -> Vec<u8>;

    fn dump(&self, stats: Vec<Self::Statement>) -> Vec<u8> {
        let mut byts = Vec::new();
        for stat in stats {
            let inst_bytes = self.to_binary(stat);
            byts.extend(inst_bytes);
        }
        byts
    }
}




//idea: convert the binary representation as specified in the riscv32 ISA into a statement
//the idea is perhaps to convert binary into bytecode?
trait BinaryDecoder {
    type Statement;

    fn to_stat(&self, byts: Vec<u8>) -> Self::Statement;
    // fn to_stat(&self, byts: u32) -> Self::Statement;

    // fn undump(&self, byts: Vec<u8>) -> Vec<Self::Statement> {
    //     let mut byts = Vec::new();
    //     for b in byts {
    //         let inst = self.to_stat(b);
    //         byts.extend(inst_bytes);
    //     }
    //     byts
    // }
}




struct Memory {
    memory: Vec<u8>,
}

impl Memory {
    fn write(&mut self, mem: Vec<u8>) -> () {
        self.memory.clear();
        self.memory.extend(mem);
    }

    fn read(&self, idx: usize) -> u32 {
        let b1: u32 = (*self.memory.get(idx).unwrap()).into();
        let b2: u32 = (*self.memory.get(idx+1).unwrap()).into();
        let b3: u32 = (*self.memory.get(idx+2).unwrap()).into();
        let b4: u32 = (*self.memory.get(idx+3).unwrap()).into();
        let mut n: u32 = 0;
        n |= b1 << 3;
        n |= b2 << 2;
        n |= b3 << 1;
        n |= b4 << 0;
        n
    }
}



struct CPU {
    registers: Vec<u32>,
    pc: usize,
}

impl CPU {
    fn write(&mut self, reg: usize, v: u32) {
        if let Some(r) = self.registers.get_mut(reg) {
            *r = v;
        }
    }
    fn read(&self, reg: usize) -> u32 {
        *self.registers.get(reg).expect("Unknown register")
    }

    fn write_pc(&mut self, v: usize) {
        self.pc = v;
    }

    fn read_pc(&self) -> usize {
        self.pc
    }
}




struct Machine {
    memory: Memory,
    cpu: CPU,
}

impl Machine {
    fn fetch(&self) -> u32 {
        self.memory.read(self.cpu.pc)
    }

    //should i execute a instruction as a u32 or as a Statement?
    fn decode(&mut self, instr: u32) -> () {
        todo!();
    }

    fn cycle(&mut self) -> () {
        let inst = self.fetch();
        self.decode(inst);
    }
}




trait Executor<'a> {
    type Statement;

    fn execute(&'a self, m: &mut Machine, inst: Self::Statement) -> ();
}

struct StatementExecutor;

impl<'a> Executor<'a> for StatementExecutor {
    type Statement = intel::Statement<'a>;
    fn execute(&'a self, m: &mut Machine, inst: Self::Statement) -> () {
        match inst {
            intel::Statement::Label(s) => {

            },
            intel::Statement::Directive(s) => {

            },
            intel::Statement::Instruction{opcode, args} => {
                match opcode {
                    intel::Command::RET => {

                    },
                    intel::Command::PSEUDO(pseudo) => {

                    },
                    intel::Command::OP(op) => {
                        match op {
                            intel::Opcode::RV32I(rv32i) => {
                                match rv32i {
                                    rustv::spec::extensions::rv32i::Opcode::ADDI => {
                                    },
                                    _ => {
                                    }
                                }
                            }
                        }
                    },
                }
            },
        }
    }
}
