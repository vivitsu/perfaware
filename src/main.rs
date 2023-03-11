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
    // file name
    filename: String,
}

impl ByteStream {
    fn new(path: &Path, filename: String) -> Result<ByteStream> {
        let bytes = read(path)?;
        Ok( ByteStream {
            bytes: bytes,
            pos: 0,
            filename,
        })
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.pos == self.bytes.len() {
            return None;
        }
        let byte = self.bytes[self.pos];
        self.pos += 1;
        Some(byte)
    }

    fn read_word(&mut self) -> Option<u16> {
        if self.pos == self.bytes.len() || self.pos + 1 == self.bytes.len() {
            return None;
        }
        let lo: u16 = self.bytes[self.pos] as u16;
        let hi: u16 = (self.bytes[self.pos + 1] as u16) << 8;
        let word = hi | lo;
        self.pos += 2;
        Some(word)
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.bytes.len()
    }

}

#[derive(PartialEq, Eq)]
enum Mode {
    Mem,    
    MemByte,
    MemWord,
    Reg,
    Imm,
    None
}

#[derive(PartialEq, Eq)]
enum Direction {
    RegSrc,
    RegDst,
    None,
}

#[derive(PartialEq, Eq)]
enum Displacement {
    Byte,
    Word,
    None,
}

#[derive(PartialEq, Eq)]
enum Opcode {
    MovRmReg,
    MovImmRm,
    MovImmReg,
    None,
}

struct InstPart<'a> {
    opcode: &'a str,
    direction: Direction,
    wide: bool,
    mode: Mode,
    reg: u8,
    rm: u8,
    // displacement, if any
    disp: i16,
    // true if are there more parts to parse for this instruction
    next: bool,
    // byte or word
    disp_type: Displacement,
}

impl<'a> InstPart<'a> {
    fn new() -> InstPart<'a> {
        Self {
            opcode: "",
            direction: Direction::None,
            wide: false,
            mode: Mode::None,            
            reg: 0,
            rm: 0,
            disp: 0,
            next: false,
            disp_type: Displacement::None,
        }
    }

    fn reset(&mut self) {
        self.opcode = "";
        self.direction = Direction::None;
        self.wide = false;
        self.mode = Mode::None;
        self.reg = 0;
        self.rm = 0;
        self.disp = 0;
        self.next = false;
        self.disp_type = Displacement::None;
    }

    fn opcode(byte: u8) -> Opcode {
        match byte & OPCODE {
            MOV_RM_REG => Opcode::MovRmReg,
            MOV_IMM_REG => Opcode::MovImmReg,
            MOV_IMM_RM => Opcode::MovImmRm,
            _ => Opcode::None,
        }
    }

    fn parse(&mut self, byte: u8) {
        /*
        match Self::opcode(byte) {
            Opcode::MovRmReg => {

            },
            Opcode::MovImmReg => todo!(),
            Opcode::MovImmRm => todo!(),
            Opcode::None => {
                // parse the second (or other) byte(s)
            },
        }
        */

        if Self::opcode(byte) == Opcode::MovRmReg {
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
            if  self.mode == Mode::None {
                self.reg = (byte & REGISTER) >> 3;
                self.rm = byte & REG_MEM as u8;
                let mode = (byte & MODE) >> 6;
                if mode == 0 {
                    if self.rm == 6 {
                        self.mode = Mode::Imm;
                        self.disp_type = Displacement::Word;
                    } else {
                        self.mode = Mode::Mem;
                        self.next = false;
                    }
                } else if mode == 1 {
                    self.mode = Mode::MemByte;
                    self.disp_type = Displacement::Byte;
                } else if mode == 2 {
                    self.mode = Mode::MemWord;
                    self.disp_type = Displacement::Word;
                } else {
                    assert!(mode == 3);
                    self.mode = Mode::Reg;
                    self.next = false;
                }
            }
        }
    }

    fn parse_disp_byte(&mut self, byte: u8) {
        self.disp = byte as i16;
        self.next = false;
    }

    fn parse_disp_word(&mut self, word: u16) {
        self.disp = word as i16;
        self.next = false;
    }

    fn into_inst(&self) -> Inst {
        let reg = if self.wide {
            WIDE_REGS[self.reg as usize].to_string()
        } else {
            BYTE_REGS[self.reg as usize].to_string()
        };

        let rm = if self.mode == Mode::Reg {
            if self.wide {
                WIDE_REGS[self.rm as usize].to_string()
            } else {
                BYTE_REGS[self.rm as usize].to_string()
            }
        } else if self.mode == Mode::Mem {
            let disp_base = MEM_DISP[self.rm as usize];
            format!("[{}]", disp_base.clone())
        } else if self.mode == Mode::Imm {
            self.disp.to_string()
        } else {
            assert!(self.mode == Mode::MemByte || self.mode == Mode::MemWord);
            let disp_base = MEM_DISP[self.rm as usize];
            format!("[{} + {}]", disp_base, self.disp.to_string().as_str())
        };

        if self.direction == Direction::RegDst {
            Inst {
                mnemonic: String::from(self.opcode),
                dst: reg.to_owned(),
                src: rm.to_owned(),
            }
        } else {
            assert!(self.direction == Direction::RegSrc);
            Inst {
                mnemonic: String::from(self.opcode),
                dst: rm.to_owned(),
                src: reg.to_owned(),
            }
        }
    }

    fn has_next(&self) -> bool {
        self.next
    }
}

struct Inst {
    mnemonic: String,
    dst: String,
    src: String,
}

impl Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}, {}", self.mnemonic, self.dst, self.src)
    }
}

// lookup tables
const BYTE_REGS: [&str; 8] = ["al", "cl", "bl", "dl", "ah", "ch", "dh", "bh"];
const WIDE_REGS: [&str; 8] = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];
const MEM_DISP: [&str; 8] = ["bx + si", "bx + di", "bp + si", "bp + di", "si", "di", "", "bx"];

// masks
const OPCODE: u8 = 0b1111_1100;
const DIRECTION: u8 = 0b0000_0010;
const WIDE_OR_BYTE: u8 = 0b0000_0001;
const MODE: u8 = 0b1100_0000;
const REGISTER: u8 = 0b0011_1000;
const REG_MEM: u8 = 0b0000_0111;

// opcodes
const MOV_RM_REG: u8 = 0b1000_1000; // register/memory to/from register
const MOV_IMM_RM: u8 = 0b1100_0110; // immediate to register/memory
const MOV_IMM_REG: u8 = 0b1011_0000; // immediate to register

fn main() -> Result<()> {
    let path = Path::new("computer_enhance/perfaware/part1/");
    let mut readers = vec![];
    let mut writers = vec![];

    if path.is_dir() {
        for (index, maybe_entry) in path.read_dir()?.enumerate() {
            let entry = maybe_entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name().into_string().expect("get filename as string");
                if filename.contains(".asm") || !filename.contains("0039") {
                    continue;
                }
                let stream = ByteStream::new(entry.path().as_path(), filename)?;
                readers.push(stream);
                let name = format!("decoded_{}", index);
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(name)
                    .expect("file created");
                writers.push(file);
            }
        }
    }

    for (index, reader) in readers.iter_mut().enumerate() {
        let writer = &mut writers[index];
        writeln!(writer, "; {}", reader.filename)?;
        writeln!(writer, "bits 16")?;
        writeln!(writer)?;

        let mut parts = InstPart::new();

        while !reader.is_empty() {
            // read first byte
            let b1 = reader.read_byte().unwrap();
            let b2 = reader.read_byte().unwrap();
            parts.parse(b1);
            parts.parse(b2);

            // read displacement bytes
            while parts.has_next() {
                if parts.disp_type == Displacement::Byte {
                    parts.parse_disp_byte(reader.read_byte().unwrap());
                } else {
                    assert!(parts.disp_type == Displacement::Word);
                    parts.parse_disp_word(reader.read_word().unwrap());
                }
            }

            let inst: Inst = parts.into_inst();
            writeln!(writer, "{}", inst.to_string())?;
            parts.reset();
        }
    }
    Ok(())
}
