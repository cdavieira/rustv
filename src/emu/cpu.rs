pub trait CPU {
    fn write(&mut self, reg: usize, v: u32) ;
    fn read(&self, reg: usize) -> u32 ;

    fn write_pc(&mut self, v: usize) ;
    fn read_pc(&self) -> usize ;

    fn read_all(&self) -> Vec<u32>;
    fn write_all(&mut self, gps: Vec<u32>, pc: usize) -> () ;
}


/* Possible implementation */

pub struct SimpleCPU {
    registers: Vec<u32>,
    pc: usize,
}

impl SimpleCPU {
    pub fn new() -> Self {
        SimpleCPU {
            registers: (0..32).map(|_| 0).collect(),
            pc: 0,
        }
    }
}

// TODO: create test to all these methods

impl CPU for SimpleCPU {
    fn write(&mut self, reg: usize, v: u32) {
        if reg == 0 {
            return;
        }
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

    fn read_all(&self) -> Vec<u32> {
        let mut state = self.registers.clone();
        state.push(self.pc as u32);
        state
    }

    fn write_all(&mut self, gps: Vec<u32>, pc: usize) -> () {
        for (idx, reg) in gps.into_iter().skip(1).enumerate() {
            self.registers[idx+1] = reg;
        }
        self.pc = pc;
    }
}
