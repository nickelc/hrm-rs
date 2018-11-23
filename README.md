# Tool to run `Human Resource Machine` programs

## Building

hrm is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
Building is easy:

```
$ git clone https://github.com/nickelc/hrm-rs.git
$ cd hrm-rs
$ cargo build --release
$ ls -l ./target/release/hrm
```

## Usage

```text
hrm < floor.asm
hrm <inbox> < floor.asm
hrm - <tiles> < floor.asm
hrm <inbox> - <tiles> < floor.asm
```

## Examples

### Put everything from the inbox to the outbox

```
a:
INBOX
OUTBOX
JUMP a
```

```text
hrm 1 2 3 4 5 < example.asm
hrm A S D F 1 2 < example.asm
hrm ASDF 1 2 < example.asm
```

### Add 3 to everything in the inbox

```
a:
INBOX
ADD 0
OUTBOX
JUMP a
```

```text
hrm 1 2 3 4 5 - 0:3 < example.asm
```

## Related Projects

- [Javascript parser](https://github.com/nrkn/hrm-parser)
- [Javascript runtime](https://github.com/nrkn/hrm-cpu)
- [Level data](https://github.com/atesgoral/hrm-level-data)
- [Solutions and speed/size hacks, exploits](https://github.com/atesgoral/hrm-solutions)
