#[macro_use]
extern crate nom;

use std::env;
use std::io;
use std::io::prelude::*;

mod error;
mod machine;
mod op;
mod parser;

use machine::{Cpu, Data, Memory};

fn main() {
    let mut inbox = vec![];
    let mut mem = Memory::new();

    let mut is_mem = false;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.parse().map(Data::Number) {
            Ok(n) => inbox.push(n),
            Err(_) => {
                if !is_mem && &arg[..] == "-" {
                    is_mem = true;
                    continue;
                }
                if !is_mem {
                    inbox.extend(
                        arg.chars()
                            .filter(char::is_ascii_alphabetic)
                            .map(|c| c.to_ascii_uppercase())
                            .map(Data::Char),
                    );
                } else {
                    let mut it = arg.split(":");
                    if let Some(Ok(tile)) = it.next().map(|s| s.parse()) {
                        if let Some(data) = it.next() {
                            match data.parse().map(Data::Number) {
                                Ok(n) => {
                                    mem.insert(tile, n);
                                }
                                Err(_) => {
                                    let mut c = data
                                        .chars()
                                        .filter(char::is_ascii_alphabetic)
                                        .map(|c| c.to_ascii_uppercase())
                                        .map(Data::Char);
                                    if let Some(c) = c.next() {
                                        mem.insert(tile, c);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut buf = String::new();
    input
        .read_to_string(&mut buf)
        .expect("failed to read stdin");

    let output = Cpu::new(&buf, mem, inbox).run().expect("program failed");
    println!("Output: {:?}", output);
}
