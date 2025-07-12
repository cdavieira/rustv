pub trait Memory {
    fn erase(&mut self) ;
    fn dump(&mut self, mem: Vec<u8>) -> () ;

    fn write(&mut self, idx: usize, v: u8) -> () ;
    fn read(&self, idx: usize) -> u8 ;

    fn write_word(&self, idx: usize, v: u32) -> () ;
    fn read_word(&self, idx: usize) -> u32 ;
}


/* Basic implementation */

pub struct BasicMemory {
    data: Vec<u8>,
}

impl BasicMemory {
    pub fn new() -> Self {
        BasicMemory { data: Vec::new() }
    }
}

impl Memory for BasicMemory {
    fn erase(&mut self) {
        self.data.clear();
    }

    fn dump(&mut self, mem: Vec<u8>) -> () {
        self.erase();
        self.data.extend(mem);
    }

    fn write(&mut self, idx: usize, v: u8) -> () {
        *self.data.get_mut(idx).unwrap() = v;
    }

    fn read(&self, idx: usize) -> u8 {
        *self.data.get(idx).unwrap()
    }

    fn write_word(&self, idx: usize, v: u32) -> () {
    }

    fn read_word(&self, idx: usize) -> u32 {
        let b1: u32 = (*self.data.get(idx).unwrap()).into();
        let b2: u32 = (*self.data.get(idx+1).unwrap()).into();
        let b3: u32 = (*self.data.get(idx+2).unwrap()).into();
        let b4: u32 = (*self.data.get(idx+3).unwrap()).into();
        let mut n: u32 = 0;
        n |= b1 << 3;
        n |= b2 << 2;
        n |= b3 << 1;
        n |= b4 << 0;
        n
    }
}
