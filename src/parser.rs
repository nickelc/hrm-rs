use std::collections::BTreeMap;

use nom::types::CompleteStr;
use nom::{digit, is_alphanumeric, is_digit, multispace, space};

use crate::op::{Addr, Instr};

#[derive(Debug, PartialEq)]
enum Token<'a> {
    Inbox,
    Outbox,
    Label(&'a str),
    CopyTo(Addr),
    CopyFrom(Addr),
    Add(Addr),
    Sub(Addr),
    BumpUp(Addr),
    BumpDown(Addr),
    Jump(&'a str),
    JumpZero(&'a str),
    JumpNeg(&'a str),
}

pub fn parse(input: &str) -> Vec<Instr> {
    let mut tokens = parse_tokens(CompleteStr(input))
        .map(|(_, res)| res)
        .unwrap_or_else(|_| Vec::new());

    let (lines, jumps) = {
        let mut jumps = BTreeMap::new();
        let mut line = vec![];
        let mut lines = vec![];
        let mut n = 0;
        while !tokens.is_empty() {
            let t = tokens.remove(0);
            let is_instr = match t {
                Token::Label(name) => {
                    jumps.insert(name, n);
                    false
                }
                _ => true,
            };
            line.push(t);
            if tokens.is_empty() || is_instr {
                lines.push(line);
                line = vec![];
                n += 1;
            }
        }
        (lines, jumps)
    };

    lines.iter().fold(Vec::new(), |mut prog, l| {
        match l.last() {
            Some(Token::Inbox) => prog.push(Instr::Inbox),
            Some(Token::Outbox) => prog.push(Instr::Outbox),
            Some(Token::CopyTo(addr)) => prog.push(Instr::CopyTo(*addr)),
            Some(Token::CopyFrom(addr)) => prog.push(Instr::CopyFrom(*addr)),
            Some(Token::Add(addr)) => prog.push(Instr::Add(*addr)),
            Some(Token::Sub(addr)) => prog.push(Instr::Sub(*addr)),
            Some(Token::BumpUp(addr)) => prog.push(Instr::BumpUp(*addr)),
            Some(Token::BumpDown(addr)) => prog.push(Instr::BumpDown(*addr)),
            Some(Token::Jump(name)) => {
                if let Some(line) = jumps.get(name) {
                    prog.push(Instr::Jump(*line));
                }
            }
            Some(Token::JumpNeg(name)) => {
                if let Some(line) = jumps.get(name) {
                    prog.push(Instr::JumpNeg(*line));
                }
            }
            Some(Token::JumpZero(name)) => {
                if let Some(line) = jumps.get(name) {
                    prog.push(Instr::JumpZero(*line));
                }
            }
            _ => {}
        }
        prog
    })
}

named!(parse_tokens<CompleteStr, Vec<Token>>,
    preceded!(hrm_sep, tokens)
);

named!(doc_comment<CompleteStr, CompleteStr>,
    recognize!(
        tuple!(
            tag!("--"),
            take_until_and_consume!("\n")
        )
    )
);

named!(hrm_sep<CompleteStr, CompleteStr>,
    recognize!(
        many0!(
            alt!(multispace | doc_comment)
        )
    )
);

named!(comment<CompleteStr, CompleteStr>,
    recognize!(
        many0!(
            tuple!(
                tag!("COMMENT"),
                space,
                take_while!(|c| is_digit(c as u8)),
                take_until_and_consume!("\n")
            )
        )
    )
);

named!(tokens<CompleteStr, Vec<Token>>,
    many0!(
        alt!(
            preceded!(comment, ws!(label)) |
            preceded!(comment, ws!(inoutbox)) |
            preceded!(comment, alt!(ws!(jump) | ws!(jumpn) | ws!(jumpz))) |
            preceded!(comment, alt!(ws!(copy_to) | ws!(copy_from))) |
            preceded!(comment, alt!(ws!(add) | ws!(sub))) |
            preceded!(comment, alt!(ws!(bump_up) | ws!(bump_down)))
        )
    )
);

named!(label<CompleteStr, Token>,
    map!(
        do_parse!(
            name: take_while!(|c| is_alphanumeric(c as u8)) >>
            tag!(":") >>
            (name)
        ),
        |s| Token::Label(s.0)
    )
);

named!(inoutbox<CompleteStr, Token>,
    do_parse!(
        inst: alt!(
            value!(Token::Inbox, tag!("INBOX")) |
            value!(Token::Outbox, tag!("OUTBOX"))
        ) >>
        (inst)
    )
);

named!(addr<CompleteStr, Addr>,
    alt!(
        do_parse!(
            v: map_res!(digit, to_usize) >>
            (Addr::Direct(v))
        ) |
        do_parse!(
            char!('[') >>
            v: map_res!(digit, to_usize) >>
            char!(']') >>
            (Addr::Ref(v))
        )
    )
);

named!(jump<CompleteStr, Token>,
    map!(
        do_parse!(
            tag!("JUMP") >>
            space >>
            to:  take_while!(|c| is_alphanumeric(c as u8)) >>
            (to)
        ),
        |s| Token::Jump(s.0)
    )
);

named!(jumpn<CompleteStr, Token>,
    map!(
        do_parse!(
            tag!("JUMPN") >>
            space >>
            to:  take_while!(|c| is_alphanumeric(c as u8)) >>
            (to)
        ),
        |s| Token::JumpNeg(s.0)
    )
);

named!(jumpz<CompleteStr, Token>,
    map!(
        do_parse!(
            tag!("JUMPZ") >>
            space >>
            to:  take_while!(|c| is_alphanumeric(c as u8)) >>
            (to)
        ),
        |s| Token::JumpZero(s.0)
    )
);

macro_rules! impl_token(
    ($func:ident, $token:ident, $tag:expr) => (
        named!($func<CompleteStr, Token>,
            do_parse!(
                tag!($tag) >>
                space >>
                a: addr >>
                (Token::$token(a))
            )
        );
    );
);

impl_token!(add, Add, "ADD");
impl_token!(sub, Sub, "SUB");
impl_token!(copy_to, CopyTo, "COPYTO");
impl_token!(copy_from, CopyFrom, "COPYFROM");
impl_token!(bump_up, BumpUp, "BUMPUP");
impl_token!(bump_down, BumpDown, "BUMPDN");

fn to_usize(input: CompleteStr) -> Result<usize, ()> {
    use std::str::FromStr;
    match FromStr::from_str(input.0) {
        Err(_) => Err(()),
        Ok(i) => Ok(i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens() {
        use super::Token::*;

        let src = r#"
        a:
            INBOX
            OUTBOX
            ADD 1
            SUB [2]
            COPYTO 3
            COPYFROM [4]
            JUMP a
        "#;

        assert_eq!(
            tokens(CompleteStr(src)),
            Ok((
                CompleteStr(""),
                vec![
                    Label("a"),
                    Inbox,
                    Outbox,
                    Add(Addr::Direct(1)),
                    Sub(Addr::Ref(2)),
                    CopyTo(Addr::Direct(3)),
                    CopyFrom(Addr::Ref(4)),
                    Jump("a"),
                ]
            ))
        );
    }

    #[test]
    fn test_addr() {
        assert_eq!(
            addr(CompleteStr("1")),
            Ok((CompleteStr(""), Addr::Direct(1)))
        );
        assert_eq!(
            addr(CompleteStr("[2]")),
            Ok((CompleteStr(""), Addr::Ref(2)))
        );
    }

    #[test]
    fn test_label() {
        assert_eq!(
            label(CompleteStr("a:")),
            Ok((CompleteStr(""), Token::Label("a")))
        );
        assert_eq!(
            label(CompleteStr("b1:")),
            Ok((CompleteStr(""), Token::Label("b1")))
        );
    }

    #[test]
    fn test_inoutbox() {
        assert_eq!(
            inoutbox(CompleteStr("INBOX")),
            Ok((CompleteStr(""), Token::Inbox))
        );
        assert_eq!(
            inoutbox(CompleteStr("OUTBOX")),
            Ok((CompleteStr(""), Token::Outbox))
        );
    }

    #[test]
    fn test_jump() {
        assert_eq!(
            jump(CompleteStr("JUMP a")),
            Ok((CompleteStr(""), Token::Jump("a")))
        );
        assert_eq!(
            jumpn(CompleteStr("JUMPN b1")),
            Ok((CompleteStr(""), Token::JumpNeg("b1")))
        );
        assert_eq!(
            jumpz(CompleteStr("JUMPZ c2")),
            Ok((CompleteStr(""), Token::JumpZero("c2")))
        );
    }

    #[test]
    fn test_add_sub() {
        assert_eq!(
            add(CompleteStr("ADD 1")),
            Ok((CompleteStr(""), Token::Add(Addr::Direct(1))))
        );
        assert_eq!(
            add(CompleteStr("ADD [2]")),
            Ok((CompleteStr(""), Token::Add(Addr::Ref(2))))
        );

        assert_eq!(
            sub(CompleteStr("SUB 1")),
            Ok((CompleteStr(""), Token::Sub(Addr::Direct(1))))
        );
        assert_eq!(
            sub(CompleteStr("SUB [2]")),
            Ok((CompleteStr(""), Token::Sub(Addr::Ref(2))))
        );
    }

    #[test]
    fn test_bump() {
        assert_eq!(
            bump_up(CompleteStr("BUMPUP 1")),
            Ok((CompleteStr(""), Token::BumpUp(Addr::Direct(1))))
        );
        assert_eq!(
            bump_up(CompleteStr("BUMPUP [2]")),
            Ok((CompleteStr(""), Token::BumpUp(Addr::Ref(2))))
        );

        assert_eq!(
            bump_down(CompleteStr("BUMPDN 1")),
            Ok((CompleteStr(""), Token::BumpDown(Addr::Direct(1))))
        );
        assert_eq!(
            bump_down(CompleteStr("BUMPDN [2]")),
            Ok((CompleteStr(""), Token::BumpDown(Addr::Ref(2))))
        );
    }

    #[test]
    fn test_copy() {
        assert_eq!(
            copy_to(CompleteStr("COPYTO 1")),
            Ok((CompleteStr(""), Token::CopyTo(Addr::Direct(1))))
        );
        assert_eq!(
            copy_to(CompleteStr("COPYTO [2]")),
            Ok((CompleteStr(""), Token::CopyTo(Addr::Ref(2))))
        );

        assert_eq!(
            copy_from(CompleteStr("COPYFROM 1")),
            Ok((CompleteStr(""), Token::CopyFrom(Addr::Direct(1))))
        );
        assert_eq!(
            copy_from(CompleteStr("COPYFROM [2]")),
            Ok((CompleteStr(""), Token::CopyFrom(Addr::Ref(2))))
        );
    }
}
