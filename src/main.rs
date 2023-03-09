use std::{
    path::Path,
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

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos == self.bytes.len() {
            return None;
        }
        let byte = self.bytes[self.pos];
        self.pos += 1;
        Some(byte)
    }    
}

enum Masks {
    OpCode = 0b1111_1100,
    Direction = 0b0000_0010,
    WideOrByte = 0b0000_0001,
    Mode = 0b1100_0000,
    Register = 0b0011_1000,
    RegOrMem = 0b0000_0111,
}

enum OpCode {
    MOV = 0b1000_1000,
}

struct InstPart<'a> {
    opcode: &'a str,
    direction: bool,
    wide: bool,
    mode: Mode,
}

enum Mode {

}

struct Inst<'a> {
    mnemonic: &'a str,
    dst: &'a str,
    src: &'a str,
}

const byte_regs: [&str; 8] = ["al", "cl", "bl", "dl", "ah", "ch", "bh", "dh"];
const wide_regs: [&str; 8] = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];

fn main() -> Result<()> {
    let path = Path::new("computer_enhance/perfaware/part1/listing_0037_single_register_mov");
    let bytes = read(path).unwrap();
    let mut decoded = OpenOptions::new().read(true).write(true).create(true).open("decoded").expect("could not create file");

    assert!(bytes.len() == 2);

    let opcode_mask: u8 = 0b1111_1100;
    let d_mask: u8 = 0b0000_0010;
    let w_mask: u8 = 0b0000_0001;
    let mod_mask: u8 = 0b1100_0000;
    let reg_mask: u8 = 0b0011_1000;
    let rm_mask: u8 = 0b0000_0111;

    let mov_opcode: u8 = 0b1000_1000;

    // Read first byte
    let b1 = bytes[0];
    let opcode = b1 & opcode_mask;
    let direction = b1 & d_mask;
    let wide_mode = b1 & w_mask;

    let b2 = bytes[1];
    let mode = b2 & mod_mask;
    let reg = b2 & reg_mask;
    let rm = b2 & rm_mask;

    writeln!(decoded, "bits 16")?;
    writeln!(decoded, "")?;

    if opcode == mov_opcode {
        write!(decoded, "mov ")?;
    }

    let mut dest = String::new();
    let mut src = String::new();

    if direction == 2 {
        // reg defines destination
        if wide_mode == 1 {
            dest = "cx".to_string(); // hard code something for now
        } else {
            dest = "cl".to_string();
        }
    } else {
        assert!(direction == 0);
        if wide_mode == 1 {
            src = "cx".to_string(); // hard code something for now
        } else {
            src = "cl".to_string();
        }
    }

    if mode == 0b1100_0000 {
        if src.is_empty() {
            if wide_mode == 1 {
                src = "ax".to_string();
            } else {
                src = "al".to_string();
            }
        } else {
            assert!(!src.is_empty());
            assert!(dest.is_empty());
            if wide_mode == 1 {
                dest = "ax".to_string();
            } else {
                dest = "al".to_string();
            }
        }
    }

    write!(decoded, "{}, {}", dest, src)?;
    writeln!(decoded)?;

    Ok(())
}
