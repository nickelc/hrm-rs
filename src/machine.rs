use std::char;
use std::collections::BTreeMap;
use std::fmt;

use error::{self, Result, RuntimeError::*};
use op::{Addr, Instr};
use parser::parse;

pub type Tile = usize;
pub type Memory = BTreeMap<Tile, Data>;

#[derive(Clone)]
pub enum Data {
    Number(i64),
    Char(char),
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Data::Number(n) => n.fmt(f),
            Data::Char(c) => c.fmt(f),
        }
    }
}

#[derive(Default)]
struct State {
    program: Vec<Instr>,
    accumulator: Option<Data>,
    memory: Memory,
    inbox: Vec<Data>,
    outbox: Vec<Data>,
    counter: usize,
    steps: usize,
    running: bool,
}

pub struct Cpu {
    state: State,
}

impl Cpu {
    pub fn new(source: &str, memory: Memory, inbox: Vec<Data>) -> Self {
        let state = State {
            program: parse(source),
            memory,
            inbox,
            ..Default::default()
        };
        Cpu { state }
    }

    pub fn run(mut self) -> Result<Vec<Data>> {
        self.state.running = self.state.counter < self.state.program.len();

        println!("RUN");
        println!("  Inbox: {:?}", self.state.inbox);
        println!("  Outbox: {:?}", self.state.outbox);
        println!("  Mem: {:?}", self.state.memory);
        while self.state.running {
            self.step()?;
        }
        Ok(self.state.outbox)
    }

    fn get_hands(&self) -> Result<Data> {
        self.state.accumulator.clone().ok_or_else(|| EmptyHands)
    }

    #[inline]
    fn get_tile_idx(&self, addr: &Addr) -> Result<Tile> {
        match addr {
            Addr::Direct(i) => Ok(*i),
            Addr::Ref(i) => match self.state.memory.get(i) {
                Some(Data::Number(n)) => Ok(*n as usize),
                Some(Data::Char(_)) => error::bad_address(*i),
                None => error::empty_tile(*i),
            },
        }
    }

    fn get_tile(&self, addr: &Addr) -> Result<(Tile, Data)> {
        let tile = self.get_tile_idx(addr)?;
        self.state
            .memory
            .get(&tile)
            .cloned()
            .map(|d| (tile, d))
            .ok_or_else(|| EmptyTile(tile))
    }

    fn step(&mut self) -> Result<()> {
        self.state.running = self.state.counter < self.state.program.len();

        if self.state.running {
            let instr = &self.state.program[self.state.counter];

            match instr {
                Instr::Inbox => {
                    if self.state.inbox.is_empty() {
                        self.state.counter = usize::max_value();
                        return Ok(());
                    }
                    self.state.accumulator = self.state.inbox.drain(0..1).next();
                }
                Instr::Outbox => {
                    let data = self.state.accumulator.take().ok_or_else(|| EmptyHands)?;
                    self.state.outbox.push(data);
                }
                Instr::CopyFrom(addr) => {
                    let (_, data) = self.get_tile(addr)?;
                    self.state.accumulator = Some(data);
                }
                Instr::CopyTo(addr) => {
                    let tile = self.get_tile_idx(addr)?;
                    let data = self.get_hands()?;
                    self.state.memory.insert(tile, data);
                }
                Instr::Add(addr) => {
                    let data = (self.get_hands()?, self.get_tile(addr).map(|(_, d)| d)?);
                    match data {
                        (Data::Number(a), Data::Number(b)) => {
                            self.state.accumulator = Some(Data::Number(a + b));
                        }
                        (_, Data::Char(_)) => {
                            return error::invalid_op("You can't ADD with a letter");
                        }
                        (Data::Char(_), _) => {
                            return error::invalid_op("You can't ADD with a letter");
                        }
                    }
                }
                Instr::Sub(addr) => {
                    let data = (self.get_hands()?, self.get_tile(addr).map(|(_, d)| d)?);
                    match data {
                        (Data::Number(a), Data::Number(b)) => {
                            self.state.accumulator = Some(Data::Number(a - b));
                        }
                        (Data::Char(a), Data::Char(b)) => {
                            let n = a as i64 - b as i64;
                            self.state.accumulator = Some(Data::Number(n));
                        }
                        (Data::Char(_), Data::Number(_)) => {
                            return error::invalid_op("You can't SUB with mixed operands");
                        }
                        (Data::Number(_), Data::Char(_)) => {
                            return error::invalid_op("You can't SUB with mixed operands");
                        }
                    }
                }
                Instr::BumpUp(addr) => {
                    let (tile, data) = self.get_tile(addr)?;
                    match data {
                        Data::Number(a) => {
                            let a = a + 1;
                            self.state.memory.insert(tile, Data::Number(a));
                            self.state.accumulator = Some(Data::Number(a));
                        }
                        Data::Char(_) => return error::invalid_op("You can't BUMP+ with a letter"),
                    }
                }
                Instr::BumpDown(addr) => {
                    let (tile, data) = self.get_tile(addr)?;
                    match data {
                        Data::Number(a) => {
                            let a = a - 1;
                            self.state.memory.insert(tile, Data::Number(a));
                            self.state.accumulator = Some(Data::Number(a));
                        }
                        Data::Char(_) => return error::invalid_op("You can't BUMP- with a letter"),
                    }
                }
                Instr::Jump(line) => {
                    self.state.counter = *line;
                }
                Instr::JumpNeg(line) => match self.state.accumulator {
                    Some(Data::Number(i)) if i < 0 => self.state.counter = *line,
                    _ => self.state.counter += 1,
                },
                Instr::JumpZero(line) => match self.state.accumulator {
                    Some(Data::Number(i)) if i == 0 => self.state.counter = *line,
                    _ => self.state.counter += 1,
                },
            }
            if !instr.is_jump() {
                self.state.counter += 1;
            }
            self.state.steps += 1;
            println!("{:?}", instr);
            println!("  Inbox: {:?}", self.state.inbox);
            println!("  Outbox: {:?}", self.state.outbox);
            println!("  Hands: {:?}", self.state.accumulator);
            println!("  Mem: {:?}", self.state.memory);
        }
        Ok(())
    }
}
