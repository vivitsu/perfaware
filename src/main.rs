use std::{
    path::Path,
    fmt::Display,
    fs::{read, OpenOptions},
    io::{Write, Result},
};

struct ByteStream {
    // the bytes backing this byte stream    
    bytes: Vec<u8>,
    // current position in the stream
    pos: usize,
}

impl ByteStream {
    pub fn new(path: &Path) -> Result<ByteStream> {
        let bytes = read(path)?;
        Ok( ByteStream {
            bytes: bytes,
            pos: 0,
        })
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.pos == self.bytes.len() {
            return None;
        }
        let byte = self.bytes[self.pos];
        self.pos += 1;
        Some(byte)
    }
}

#[derive(PartialEq, Eq)]
enum Mode {
    Mem,    
    MemByte,
    MemWord,
    Reg,
    None
}

#[derive(PartialEq, Eq)]
enum Direction {
    RegSrc,
    RegDst,
}

struct InstPart<'a> {
    opcode: &'a str,
    direction: Direction,
    wide: bool,
    mode: Mode,
    reg: u8,
    rm: u8,
    disp: u16,
    // true if are there more parts to parse for this instruction
    next: bool,
}

impl<'a> InstPart<'a> {
    fn new() -> InstPart<'a> {
        Self {
            opcode: "",
            direction: Direction::RegSrc,
            wide: false,
            mode: Mode::None,            
            reg: 0,
            rm: 0,
            disp: 0,
            next: false,
        }
    }

    fn parse(&mut self, byte: u8) {
        if byte & OPCODE == MOV {
            self.opcode = "mov";
            // mov is a multi-byte instruction
            self.next = true;
            let dir_bit = (byte & DIRECTION) >> 1;
            if dir_bit == 1 {
                self.direction = Direction::RegDst;
            } else {
                assert!(dir_bit == 0);
                self.direction = Direction::RegSrc;
            }
            let wide_bit = byte & WIDE_OR_BYTE;
            if wide_bit == 1 {
                self.wide = true;
            } else {
                assert!(wide_bit == 0);
                self.wide = false;
            }
        } else {
            if self.mode == Mode::None {
                let mode = (byte & MODE) >> 6;
                if mode == 3 {
                    self.mode = Mode::Reg;
                    self.next = false;
                }
                self.reg = (byte & REGISTER) >> 3;
                self.rm = byte & REG_MEM as u8;
                // only handle reg mode for now
            }
            // only handle 2 byte mov instructions for now
        }
    }

    fn into_inst(&self) -> Inst {
        let mut reg = "";
        let mut rm = "";
        let mut dst = "";
        let mut src = "";
        if self.wide {
            reg = WIDE_REGS[self.reg as usize];
            if self.mode == Mode::Reg {
                rm = WIDE_REGS[self.rm as usize];
            }
        } else {
            assert!(!self.wide);
            reg = BYTE_REGS[self.reg as usize];
            if self.mode == Mode::Reg {
                rm = BYTE_REGS[self.rm as usize];
            }
        }

        if self.direction == Direction::RegDst {
            dst = reg;
            src = rm;
        } else {
            assert!(self.direction == Direction::RegSrc);
            src = reg;
            dst = rm;
        }

        Inst {
            mnemonic: self.opcode,
            dst,
            src
        }        
    }

    fn has_next(&self) -> bool {
        self.next
    }
}

struct Inst<'a> {
    mnemonic: &'a str,
    dst: &'a str,
    src: &'a str,
}

impl<'a> Display for Inst<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}, {}", self.mnemonic, self.dst, self.src)
    }
}

// register lookup table
const BYTE_REGS: [&str; 8] = ["al", "cl", "bl", "dl", "ah", "ch", "bh", "dh"];
const WIDE_REGS: [&str; 8] = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];

// masks
const OPCODE: u8 = 0b1111_1100;
const DIRECTION: u8 = 0b0000_0010;
const WIDE_OR_BYTE: u8 = 0b0000_0001;
const MODE: u8 = 0b1100_0000;
const REGISTER: u8 = 0b0011_1000;
const REG_MEM: u8 = 0b0000_0111;

// opcodes
const MOV: u8 = 0b1000_1000;

fn main() -> Result<()> {
    let path = Path::new("computer_enhance/perfaware/part1/listing_0038_many_register_mov");
    let mut stream = ByteStream::new(path).expect("could not open file");
    let mut decoded = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("decoded")
        .expect("could not create file");


    writeln!(decoded, "bits 16")?;
    writeln!(decoded)?;

    // indicates if we should start processing bytes as a new instruction
    let mut is_new = true;
    let mut parts = InstPart::new();

    loop {
        if let Some(byte) = stream.read_byte() {
            if is_new {
                // extract opcode, etc
                parts = InstPart::new(); // todo - avoid 
                parts.parse(byte);
                is_new = false;
                continue;
            } else {
                parts.parse(byte);
                if !parts.has_next() {
                    let inst: Inst = parts.into_inst();                    
                    writeln!(decoded, "{}", inst.to_string())?;
                    is_new = true
                }
                continue;
            }
        }
        break;
    }
    Ok(())
}
