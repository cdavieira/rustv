pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod assembler;
pub mod memory;
pub mod elf;
pub mod cpu;
pub mod machine;
pub mod utils;
pub mod elfwriter;

use crate::machine::{Machine, SimpleMachine};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let mut v: Vec<u32> = Vec::new();
    v.reserve(1000);
    let m = SimpleMachine::new(&v);
    let emu = Arc::new(Mutex::new(m));
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    println!("GDB RSP server listening on 127.0.0.1:1234...");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let emu = Arc::clone(&emu);
        println!("Got new TCP stream coming up");
        thread::spawn(move || handle_client(stream, emu));
    }
}

/// --- Emulator ---

/// --- Helper: packet encoding/decoding ---
fn compute_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

fn parse_packet(buf: &[u8]) -> Option<&[u8]> {
    if buf.starts_with(b"$") {
        if let Some(pos) = buf.iter().position(|&b| b == b'#') {
            return Some(&buf[1..pos]);
        }
    }
    None
}

/// --- GDB RSP handler ---
fn handle_client(mut stream: TcpStream, emu: Arc<Mutex<SimpleMachine>>) {
    let mut buf = [0u8; 1024];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("\nGDB:");
                for b in &buf[..n] {
                    print!("{}", b.escape_ascii().to_string());
                }
                println!("");

                if let Some(packet) = parse_packet(&buf[..n]) {
                    // Ack
                    stream.write_all(b"+").unwrap();

                    let cmd = std::str::from_utf8(packet).unwrap_or("");
                    let response = {
                        let mut emu = emu.lock().unwrap();
                        if cmd == "?" {
                            String::from("S05") // stopped with SIGTRAP
                        } else if cmd == "g" {
                            hex::encode(emu.read_registers())
                        } else if cmd.starts_with("m") {
                            // mADDR,LEN
                            let parts: Vec<&str> = cmd[1..].split(',').collect();
                            if parts.len() == 2 {
                                if let (Ok(addr), Ok(len)) = (usize::from_str_radix(parts[0], 16), usize::from_str_radix(parts[1], 16)) {
                                    if let Some(mem) = emu.read_memory(addr, len) {
                                        hex::encode(mem)
                                    } else {
                                        String::from("E01")
                                    }
                                } else { String::from("E01") }
                            } else { String::from("E01") }
                        } else if cmd == "s" {
                            emu.decode();
                            String::from("S05")
                        } else if cmd == "c" {
                            // Minimal: run one step then stop
                            emu.decode();
                            String::from("S05")
                        } else {
                            String::new() // unimplemented
                        }
                    };

                    // Send response with checksum
                    let cksum = compute_checksum(response.as_bytes());
                    let reply = format!("${}#{:02x}", response, cksum);
                    let replystr = reply.as_bytes();
                    stream.write_all(replystr).unwrap();
                    println!("Response:");
                    for b in replystr {
                        print!("{}", b.escape_ascii().to_string());
                    }
                    println!("");
                }
            }
            Err(_) => break,
        }
    }
}

