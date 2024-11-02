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
struct Mailbox([u16; 100]);
impl Mailbox {
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
                let mut s: [u16; 100] = [0; 100];
                let new_slice = try_cast_slice::<u8, u16>(buffer.as_slice());
                match new_slice {
                    Ok(new_slice) => {
                        s.copy_from_slice(new_slice);
                        Ok(Self(s))
                    }
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
        if !(0..=99).contains(&index) {
            panic!("There are only 100 mailbox (0-99) addresses available")
        }
        &self.0[index]
    }
}
impl IndexMut<usize> for Mailbox {
    fn index_mut(&mut self, index: usize) -> &mut u16 {
        if !(0..=99).contains(&index) {
            panic!("There are only 100 mailbox (0-99) addresses available")
        }
        &mut self.0[index]
    }
}
impl From<Vec<u16>> for Mailbox {
    fn from(vec: Vec<u16>) -> Self {
        let mut s: [u16; 100] = [0; 100];
        for (i, v) in vec.iter().enumerate() {
            s[i] = *v;
        }
        Self(s)
    }
}
struct Runtime {
    accumulator: u16,
    program_counter: u16,
    negative_flag: bool,
    mailbox: Mailbox,
}

impl Runtime {
    fn wrap_between_valid_values(value: u16) -> u16 {
        if value > 999 {
            value - 1000
        } else {
            value
        }
    }
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
                    let new_value = self.accumulator + self.get_addresses(addr);
                    self.accumulator = Self::wrap_between_valid_values(new_value);
                    self.negative_flag = false;
                } // Should overflow result in a negative flag?
                OpCode::SUB(addr) => {
                    let current_box = self.get_addresses(addr);
                    if (self.accumulator < current_box) {
                        self.negative_flag = true;
                        self.accumulator = current_box - self.accumulator;
                    } else {
                        self.accumulator -= current_box;
                    }
                }
                OpCode::STA(addr) => self.mailbox[addr as usize] = self.accumulator,
                OpCode::LDA(addr) => self.accumulator = self.mailbox[addr as usize],
                OpCode::BRA(addr) => self.program_counter = addr,
                OpCode::BRZ(addr) => {
                    if self.accumulator == 0 && !self.negative_flag {
                        // Should the negative flag be taken into account?
                        self.program_counter = addr;
                    }
                }
                OpCode::BRP(addr) => {
                    if !self.negative_flag {
                        self.program_counter = addr;
                    }
                }
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
                OpCode::COB(_) => break,
            }
        }
    }
}
fn main() {
    let mailbox = Mailbox::from(vec![901_u16, 308, 901, 309, 508, 209, 902, 000]);
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
