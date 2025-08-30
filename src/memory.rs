use std::io;

// TODO: add a way to deal with the endianness

pub trait Memory {
    fn reserve_bytes(&mut self, sz: usize) ;
    fn reserve_words(&mut self, sz: usize) ;
    fn clear(&mut self) ;

    fn bytes(&self) -> Vec<u8>;
    fn words(&self) -> Vec<u32>;

    fn write_file(&self, filename: &str) -> io::Result<()> ;
    fn read_file(&mut self, filename: &str) -> io::Result<()> ;

    fn read_byte(&self, idx: usize) -> u8 ;
    fn write_byte(&mut self, idx: usize, v: u8) -> () ;

    fn read_word(&self, idx: usize) -> u32 ;
    fn write_word(&mut self, idx: usize, v: u32) -> () ;

    fn read_bytes(&self, start_addr: usize, count: usize) -> Vec<u8> ;
    fn write_bytes(&mut self, start_addr: usize, data: &Vec<u8>) -> () {
        for (idx, i) in data.iter().enumerate() {
            self.write_byte(start_addr + idx, *i);
        }
    }

    fn read_words(&self, start_addr: usize, count: usize) -> Vec<u32> ;
    fn write_words(&mut self, start_addr: usize, data: &Vec<u32>) -> () {
        for (idx, i) in data.iter().enumerate() {
            self.write_word(start_addr + idx, *i);
        }
    }
}



/* Basic implementation */

use std::fs;

use object::ReadRef;

pub struct BasicMemory {
    data: Vec<u8>,
}

impl BasicMemory {
    pub fn new() -> Self {
        BasicMemory { data: Vec::new() }
    }
}

// TODO: create test to all these methods

impl Memory for BasicMemory {
    fn reserve_bytes(&mut self, sz: usize)  {
        self.data.resize(sz, 0);
    }

    fn reserve_words(&mut self, sz: usize)  {
        self.data.resize(sz * 4, 0);
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn words(&self) -> Vec<u32> {
        self.data
            .chunks_exact(4)
            .enumerate()
            .map(|(idx, _)| self.read_word(idx*4))
            .collect()
    }

    fn write_file(&self, filename: &str) -> io::Result<()> {
        fs::write(filename, &self.data)
    }

    fn read_file(&mut self, filename: &str) -> io::Result<()>  {
        let data = fs::read(filename)?;
        self.write_bytes(0, &data);
        Ok(())
    }

    fn read_byte(&self, idx: usize) -> u8 {
        *self.data.get(idx).unwrap()
    }

    fn write_byte(&mut self, idx: usize, v: u8) -> () {
        println!("Writing value {} at {} address", v, idx);
        // *self.data.get_mut(idx).unwrap() = v;
        if idx < self.data.len() {
            self.data[idx] = v;
        }
        else {
            println!("Address out of boundaries");
        }
    }

    // TODO: maybe return an option to indicate that read_word failed
    fn read_word(&self, idx: usize) -> u32 {
        if idx < self.data.len() {
            let idx: u64 = idx
                .try_into()
                .expect("failed when reading word: usize to u64");
            let s = self.data
                .read_slice_at(idx, 4)
                .expect("failed when reading word: idx out of boundaries");
            let bytes: [u8; 4] = s
                .try_into()
                .expect("failed when reading word: conversion to [u8]");
            let word = u32::from_be_bytes(bytes);
            word
        }
        else {
            0
        }
    }

    fn write_word(&mut self, idx: usize, v: u32) -> () {
        let bytes = u32::to_be_bytes(v);
        *self.data.get_mut(idx).unwrap() = bytes[0];
        *self.data.get_mut(idx+1).unwrap() = bytes[1];
        *self.data.get_mut(idx+2).unwrap() = bytes[2];
        *self.data.get_mut(idx+3).unwrap() = bytes[3];
    }

    fn read_bytes(&self, start_addr: usize, count: usize) -> Vec<u8> {
        println!("Reading {} bytes starting at address {}", count, start_addr);
        let end_addr = start_addr + count;
        if end_addr < self.data.len() {
            let start_addr: u64 = start_addr
                .try_into()
                .expect("failed when reading bytes: usize to u64");
            let s = self.data
                .read_slice_at(start_addr, count)
                .unwrap();
            s.to_vec()
        }
        else {
            Vec::new()
        }
    }

    fn write_bytes(&mut self, start_addr: usize, data: &Vec<u8>) -> () {
        for (idx, i) in data.iter().enumerate() {
            self.write_byte(start_addr + idx, *i);
        }
    }

    fn read_words(&self, start_addr: usize, count: usize) -> Vec<u32> {
        let end_addr = start_addr + (count * 4);
        let bytes = &self.data[start_addr .. end_addr];
        bytes
            .chunks_exact(4)
            .map(
                |chunk| {
                    let word: [u8; 4] = chunk
                        .try_into()
                        .expect("read_words failed when converting a chunk to a slice of 4 bytes");
                    u32::from_be_bytes(word)
                }
            )
            .collect()
    }

    fn write_words(&mut self, start_addr: usize, data: &Vec<u32>) -> () {
        for (idx, i) in data.iter().enumerate() {
            self.write_word(start_addr + 4*idx, *i);
        }
    }
}
