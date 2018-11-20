use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Addr {
    Direct(usize),
    Ref(usize),
}

#[derive(PartialEq)]
pub enum Instr {
    Inbox,
    Outbox,
    CopyTo(Addr),
    CopyFrom(Addr),
    Add(Addr),
    Sub(Addr),
    BumpUp(Addr),
    BumpDown(Addr),
    Jump(usize),
    JumpZero(usize),
    JumpNeg(usize),
}

impl Instr {
    pub fn is_jump(&self) -> bool {
        match *self {
            Instr::Jump(_) | Instr::JumpNeg(_) | Instr::JumpZero(_) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instr::Inbox => write!(f, "INBOX"),
            Instr::Outbox => write!(f, "OUTBOX"),
            Instr::CopyTo(Addr::Direct(n)) => write!(f, "COPYTO {}", n),
            Instr::CopyTo(Addr::Ref(n)) => write!(f, "COPYTO [{}]", n),
            Instr::CopyFrom(Addr::Direct(n)) => write!(f, "COPYFROM {}", n),
            Instr::CopyFrom(Addr::Ref(n)) => write!(f, "COPYFROM [{}]", n),
            Instr::Add(Addr::Direct(n)) => write!(f, "ADD {}", n),
            Instr::Add(Addr::Ref(n)) => write!(f, "ADD [{}]", n),
            Instr::Sub(Addr::Direct(n)) => write!(f, "SUB {}", n),
            Instr::Sub(Addr::Ref(n)) => write!(f, "SUB [{}]", n),
            Instr::BumpUp(Addr::Direct(n)) => write!(f, "BUMPUP {}", n),
            Instr::BumpUp(Addr::Ref(n)) => write!(f, "BUMPUP [{}]", n),
            Instr::BumpDown(Addr::Direct(n)) => write!(f, "BUMPDN {}", n),
            Instr::BumpDown(Addr::Ref(n)) => write!(f, "BUMPDN [{}]", n),
            Instr::Jump(n) => write!(f, "JUMP {}", n),
            Instr::JumpNeg(n) => write!(f, "JUMPN {}", n),
            Instr::JumpZero(n) => write!(f, "JUMPZ {}", n),
        }
    }
}
