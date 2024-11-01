use bytemuck::checked::{cast_slice, try_cast_slice, CheckedCastError};
use std::fs;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::ops::{Index, IndexMut};

macro_rules! mnemonics_type_enum {
    ($($name:ident),*)=>{
        #[derive(Debug)]
        enum MemonicType{
            $(
                $name,
            )*
        }
    }
}
macro_rules! op_code_enum {
    ($($name:ident),*)=>{
        #[derive(Debug)]
        enum OpCode{
            $(
                $name(u16),
            )*
        }
    }
}

mnemonics_type_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB);
op_code_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB);

impl From<u16> for OpCode {
    fn from(code: u16) -> Self {
        if (100..=199).contains(&code) {
            OpCode::ADD(code - 100)
        } else if (200..=299).contains(&code) {
            OpCode::SUB(code - 200)
        } else if (300..=399).contains(&code) {
            OpCode::STA(code - 300)
        } else if (500..=599).contains(&code) {
            OpCode::LDA(code - 500)
        } else if (600..=699).contains(&code) {
            OpCode::BRA(code - 600)
        } else if (700..=799).contains(&code) {
            OpCode::BRZ(code - 700)
        } else if (800..=899).contains(&code) {
            OpCode::BRP(code - 700)
        } else if code == 901 {
            OpCode::INP(901)
        } else if code == 902 {
            OpCode::OUT(902)
        } else if code == 000 {
            OpCode::HLT(000)
        } else {
            panic!("Unknown numeric code")
        }
    }
}
#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Cast(CheckedCastError),
}
struct Token {
    left: Option<String>,
    statement: MemonicType,
    right: Option<String>,
}
#[derive(Debug)]
struct Mailbox(Vec<u16>);
impl Mailbox {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn with_capacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }
    fn push(&mut self, value: u16) {
        assert!(self.0.len() >= 99);
        self.0.push(value);
    }
    fn export_to_file(&self, file: &mut File) -> Result<(), Error> {
        match file.write_all(cast_slice::<u16, u8>(self.0.as_slice())) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Io(e)),
        }
    }
    fn read_from_file(file: &mut File) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Ok(_) => {
                let new_slice = try_cast_slice::<u8, u16>(buffer.as_slice());
                match new_slice {
                    Ok(new_slice) => Ok(Self::from(new_slice.to_vec())),
                    Err(e) => Err(Error::Cast(e)),
                }
            }
            Err(e) => Err(Error::Io(e)),
        }
    }
}
impl Index<usize> for Mailbox {
    type Output = u16;
    fn index(&self, index: usize) -> &u16 {
        if (index > 99) {
            panic!("There are only 100 mailbox (0-99) addresses available")
        }
        &self.0[index]
    }
}
impl IndexMut<usize> for Mailbox {
    fn index_mut(&mut self, index: usize) -> &mut u16 {
        &mut self.0[index]
    }
}
impl From<Vec<u16>> for Mailbox {
    fn from(mailbox: Vec<u16>) -> Self {
        Self(mailbox)
    }
}
struct Runtime {
    accumulator: u16,
    program_counter: u16,
    negative_flag: bool,
    mailbox: Mailbox,
}

impl Runtime {
    fn get_addresses(&self, addr: u16) -> u16 {
        self.mailbox[addr as usize]
    }
    fn new(mailbox: Mailbox) -> Self {
        Self {
            accumulator: 0,
            program_counter: 0,
            negative_flag: false,
            mailbox,
        }
    }
    fn start(&mut self) {
        let stdin = std::io::stdin();
        loop {
            let current_instruction = OpCode::from(self.get_addresses(self.program_counter));
            self.program_counter += 1;
            match current_instruction {
                OpCode::ADD(addr) => {
                    self.accumulator += self.get_addresses(addr);
                    self.negative_flag = false;
                } // Should overflow result in a negative flag?
                OpCode::SUB(addr) => {
                    if (self.accumulator < self.get_addresses(addr)) {
                        self.negative_flag = true;
                    }
                    self.accumulator -= self.get_addresses(addr);
                }
                OpCode::STA(addr) => self.mailbox[addr as usize] = self.accumulator,
                OpCode::LDA(addr) => self.accumulator = self.mailbox[addr as usize],
                OpCode::OUT(_) => println!("{}", self.accumulator),
                OpCode::INP(_) => {
                    let mut line = String::new();
                    {
                        let mut lock = stdin.lock();
                        lock.read_line(&mut line).unwrap();
                    }
                    self.accumulator = line.trim().parse::<u16>().expect("Input must be a number");
                }
                OpCode::HLT(_) => break,
                _ => {
                    println!("TODO");
                    break;
                }
            }
        }
    }
}
fn main() {
    let mailbox = Mailbox::from(vec![504, 105, 902, 0, 2, 3]);
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("mailbox.bin")
            .expect("Failed to create file");
        mailbox.export_to_file(&mut file).unwrap();
    } // just to test the exporting and importing functionality
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open("mailbox.bin")
        .expect("Failed to open file");
    let test = Mailbox::read_from_file(&mut file).unwrap();
    println!("{:?}", test);
    let mut r = Runtime::new(mailbox);
    r.start();
}
