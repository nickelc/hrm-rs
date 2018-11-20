#[macro_use]
extern crate nom;

use std::io;
use std::io::prelude::*;

mod error;
mod machine;
mod op;
mod parser;

use machine::{Cpu, Data, Memory};

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut buf = String::new();
    input
        .read_to_string(&mut buf)
        .expect("failed to read stdin");

    let mem = Memory::new();
    let inbox = vec![Data::Number(1), Data::Number(-3)];
    let output = Cpu::new(&buf, mem, inbox).run().expect("program failed");
}
