pub trait CPU {
    fn write(&mut self, reg: usize, v: u32) ;
    fn read(&self, reg: usize) -> u32 ;

    fn write_pc(&mut self, v: usize) ;
    fn read_pc(&self) -> usize ;
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

    pub fn info(&self) -> () {
        for (i, v) in self.registers.iter().enumerate() {
            println!("Reg {}: {}", i, v);
        }
    }
}

impl CPU for SimpleCPU {
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
