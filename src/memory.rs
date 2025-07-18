pub trait Memory {
    fn erase(&mut self) ;

    fn write_byte(&mut self, idx: usize, v: u8) -> () ;
    fn read_byte(&self, idx: usize) -> u8 ;
    // fn append_byte(&mut self, v: u8) -> () ;
    fn append_bytes(&mut self, mem: Vec<u8>) -> () ;

    fn write_word(&mut self, idx: usize, v: u32) -> () ;
    fn read_word(&self, idx: usize) -> u32 ;
    fn append_word(&mut self, v: u32) -> () ;
    fn append_words(&mut self, insts: Vec<u32>) -> () {
        for inst in insts {
            self.append_word(inst);
        }

    }

    fn dump(&self, filename: &str) -> io::Result<()> ;
}

/* Basic implementation */

use std::fs;
use std::io;

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

    fn append_bytes(&mut self, mem: Vec<u8>) -> () {
        self.erase();
        self.data.extend(mem);
    }

    fn write_byte(&mut self, idx: usize, v: u8) -> () {
        *self.data.get_mut(idx).unwrap() = v;
    }

    fn read_byte(&self, idx: usize) -> u8 {
        *self.data.get(idx).unwrap()
    }

    fn write_word(&mut self, idx: usize, v: u32) -> () {
        let b1: u8 = ((v & 0b11111111_00000000_00000000_00000000) >> 24).try_into().unwrap();
        let b2: u8 = (v & 0b00000000_11111111_00000000_00000000).try_into().unwrap();
        let b3: u8 = (v & 0b00000000_00000000_11111111_00000000).try_into().unwrap();
        let b4: u8 = (v & 0b00000000_00000000_00000000_11111111).try_into().unwrap();
        *self.data.get_mut(idx).unwrap() = b1;
        *self.data.get_mut(idx+1).unwrap() = b2;
        *self.data.get_mut(idx+2).unwrap() = b3;
        *self.data.get_mut(idx+3).unwrap() = b4;
    }

    fn read_word(&self, idx: usize) -> u32 {
        let b1: u32 = (*self.data.get(idx).unwrap()).into();
        let b2: u32 = (*self.data.get(idx+1).unwrap()).into();
        let b3: u32 = (*self.data.get(idx+2).unwrap()).into();
        let b4: u32 = (*self.data.get(idx+3).unwrap()).into();
        let mut n: u32 = 0;
        n |= (b1 << 24) & 0b11111111_00000000_00000000_00000000;
        n |= (b2 << 16) & 0b00000000_11111111_00000000_00000000;
        n |= (b3 << 8)  & 0b00000000_00000000_11111111_00000000;
        n |= (b4 << 0)  & 0b00000000_00000000_00000000_11111111;
        n
    }

    fn append_word(&mut self, v: u32) -> ()  {
        let b1: u8 = ((v & 0b11111111_00000000_00000000_00000000) >> 24).try_into().unwrap();
        let b2: u8 = ((v & 0b00000000_11111111_00000000_00000000) >> 16).try_into().unwrap();
        let b3: u8 = ((v & 0b00000000_00000000_11111111_00000000) >> 8).try_into().unwrap();
        let b4: u8 = ((v & 0b00000000_00000000_00000000_11111111) >> 0).try_into().unwrap();
        self.data.push(b1);
        self.data.push(b2);
        self.data.push(b3);
        self.data.push(b4);
    }

    fn dump(&self, filename: &str) -> io::Result<()> {
        fs::write(filename, &self.data)
    }
}
