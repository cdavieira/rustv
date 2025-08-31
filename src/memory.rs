use std::io;

// TODO: add a way to deal with the endianness

pub trait Memory {
    fn endianness(&self) -> DataEndianness;

    fn bytes_count(&self) -> usize;
    fn words_count(&self) -> usize;

    fn reserve_bytes(&mut self, sz: usize) ;
    fn reserve_words(&mut self, sz: usize) ;
    fn clear(&mut self) ;

    fn bytes(&self) -> Vec<u8>;
    fn words(&self) -> Vec<u32>;

    fn write_file(&self, filename: &str) -> io::Result<()> ;
    fn read_file(&mut self, filename: &str) -> io::Result<()> ;

    fn read_byte(&self, idx: usize) -> u8 ;
    fn write_byte(&mut self, idx: usize, v: u8) -> () ;

    fn read_word(&self, idx: usize, res_endian: DataEndianness) -> u32 ;
    fn write_word(&mut self, idx: usize, val: u32) -> () ;

    fn read_bytes(&self, start_addr: usize, count: usize, res_endian: DataEndianness, alignment: usize) -> Vec<u8> ;
    fn write_bytes(&mut self, start_addr: usize, data: &Vec<u8>) -> () {
        for (idx, i) in data.iter().enumerate() {
            self.write_byte(start_addr + idx, *i);
        }
    }

    fn read_words(&self, start_addr: usize, count: usize, res_endian: DataEndianness) -> Vec<u32> ;
    fn write_words(&mut self, start_addr: usize, data: &Vec<u32>) -> () {
        for (idx, i) in data.iter().enumerate() {
            self.write_word(start_addr + idx, *i);
        }
    }
}



/* Basic implementation */

use std::fs;

use object::ReadRef;

use crate::utils::{DataEndianness, swap_chunk_endianness};

pub struct SimpleMemory {
    data: Vec<u8>,
    endianness: DataEndianness,
}

impl SimpleMemory {
    pub fn new(endianness: DataEndianness) -> Self {
        SimpleMemory { data: Vec::new(), endianness }
    }
}

// TODO: create test to all these methods

// TODO: how to check the boundaries of memory when reading/writing?

impl Memory for SimpleMemory {
    fn endianness(&self) -> DataEndianness {
        self.endianness
    }

    fn bytes_count(&self) -> usize {
        self.data.len()
    }

    fn words_count(&self) -> usize {
        self.data.len() >> 2
    }

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
            .map(|(idx, _)| self.read_word(idx*4, self.endianness))
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
        println!("{}", self.data.len());
        if idx < self.data.len() {
            self.data[idx] = v;
        }
        else {
            println!("Address out of boundaries");
        }
    }

    // TODO: maybe return an option to indicate that read_word failed
    fn read_word(&self, idx: usize, res_endian: DataEndianness) -> u32 {
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

            let word_in_memory = self.endianness.from_bytes_to_word(bytes);
            self.endianness.change_endian_word_to_word(word_in_memory, res_endian)
        }
        else {
            0
        }
    }

    // TODO: make this safe in terms of index access
    fn write_word(&mut self, idx: usize, val: u32) -> () {
        // println!("{:x} written in mem as {:?}", v, bytes);
        let values = self.endianness.from_word_to_bytes(val);
        let bytes_buffer = self.data.get_mut(idx..idx+4).unwrap();
        bytes_buffer[0] = values[0];
        bytes_buffer[1] = values[1];
        bytes_buffer[2] = values[2];
        bytes_buffer[3] = values[3];
    }

    fn read_bytes(&self, start_addr: usize, count: usize, res_endian: DataEndianness, alignment: usize) -> Vec<u8> {
        println!("Reading {} bytes starting at address {}", count, start_addr);
        let data_len = self.data.len();
        if start_addr < data_len {
            let max_count = data_len - start_addr;
            let start_addr: u64 = start_addr
                .try_into()
                .expect("failed when reading bytes: usize to u64");
            let count = if count < max_count { count } else { max_count };
            let bytes = self.data
                .read_slice_at(start_addr, count)
                .unwrap()
                .to_vec();
            if res_endian != self.endianness && alignment > 1 {
                swap_chunk_endianness(&bytes, alignment)
            }
            else {
                bytes
            }
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

    fn read_words(&self, start_addr: usize, count: usize, res_endian: DataEndianness) -> Vec<u32> {
        let end_addr = start_addr + (count * 4);
        let bytes = &self.data[start_addr .. end_addr];
        bytes
            .chunks_exact(4)
            .map(
                |chunk| {
                    let word: [u8; 4] = chunk
                        .try_into()
                        .expect("read_words failed when converting a chunk to a slice of 4 bytes");
                    self.endianness.change_endian_bytes_to_word(word, res_endian)
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
