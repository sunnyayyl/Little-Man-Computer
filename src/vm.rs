use crate::{MemonicType, OpCode};
use bytemuck::checked::{cast_slice, try_cast_slice, CheckedCastError};
use std::fs::File;
use std::io::{stdin, BufRead, Read, Write};
use std::ops::{Index, IndexMut};
#[derive(Debug)]
pub enum MailboxError {
    Io(std::io::Error),
    Cast(CheckedCastError),
}
#[derive(Debug)]
pub struct Mailbox(pub [u16; 100]);

impl Mailbox {
    pub fn set_instruction(&mut self, index: u16, p0: MemonicType, p1: Option<u16>) {
        self.set(
            index,
            OpCode::from_mnemonic_type(p0, p1).to_numeric_representation(),
        )
    }
    pub fn export_to_file(&self, file: &mut File) -> Result<(), MailboxError> {
        match file.write_all(cast_slice::<u16, u8>(self.0.as_slice())) {
            Ok(_) => Ok(()),
            Err(e) => Err(MailboxError::Io(e)),
        }
    }
    pub fn read_from_file(file: &mut File) -> Result<Self, MailboxError> {
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
                    Err(e) => Err(MailboxError::Cast(e)),
                }
            }
            Err(e) => Err(MailboxError::Io(e)),
        }
    }
    pub fn set(&mut self, index: u16, value: u16) {
        if !(0..=99).contains(&index) {
            panic!("There are only 100 mailbox (0-99) addresses available")
        }
        if value > 999 {
            panic!("Mailbox values must be between 0 and 999")
        }
        self.0[index as usize] = value;
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
pub struct Runtime {
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
    pub fn new(mailbox: Mailbox) -> Self {
        Self {
            accumulator: 0,
            program_counter: 0,
            negative_flag: false,
            mailbox,
        }
    }
    pub fn evaluate_next(&mut self) -> bool {
        // Return true to continue, false if ended (by HLT or COB)
        let current_instruction = OpCode::try_from(self.get_addresses(self.program_counter));
        if let Ok(current_instruction) = current_instruction {
            self.program_counter += 1;
            match current_instruction {
                OpCode::ADD(addr) => {
                    let new_value = self.accumulator
                        + self.get_addresses(addr.expect("ADD requires an address"));
                    self.accumulator = Self::wrap_between_valid_values(new_value);
                    self.negative_flag = false;
                } // Should overflow result in a negative flag?
                OpCode::SUB(addr) => {
                    let current_box = self.get_addresses(addr.expect("SUB requires an address"));
                    if self.accumulator < current_box {
                        self.negative_flag = true;
                        self.accumulator = current_box - self.accumulator;
                    } else {
                        self.accumulator -= current_box;
                    }
                }
                OpCode::STA(addr) => {
                    self.mailbox[addr.expect("STA requires an address") as usize] = self.accumulator
                }
                OpCode::LDA(addr) => {
                    self.accumulator = self.mailbox[addr.expect("LDA required an address") as usize]
                }
                OpCode::BRA(addr) => self.program_counter = addr.expect("BRA require an addresses"),
                OpCode::BRZ(addr) => {
                    if self.accumulator == 0 && !self.negative_flag {
                        // Should the negative flag be taken into account?
                        self.program_counter = addr.expect("BRZ require an addresses");
                    }
                }
                OpCode::BRP(addr) => {
                    if !self.negative_flag {
                        self.program_counter = addr.expect("BRP require an addresses");
                    }
                }
                OpCode::OUT(_) => println!("{}", self.accumulator),
                OpCode::INP(_) => {
                    let mut line = String::new();
                    {
                        let mut lock = stdin().lock();
                        lock.read_line(&mut line).unwrap();
                    }
                    self.accumulator = line.trim().parse::<u16>().expect("Input must be a number");
                }
                OpCode::HLT(_) => return false,
                OpCode::COB(_) => return false,
                OpCode::DAT(_) => return false, //should DAT be treated as end of program?
                OpCode::SOUT(_) => {
                    let char = u8::try_from(self.accumulator)
                        .expect("Cannot be converted to ascii character")
                        as char;
                    print!("{}", char);
                }
            }
            true
        } else {
            println!("Invalid instruction at address {}", self.program_counter);
            println!("{:?}", self.mailbox);
            false
        }
    }
    pub fn start(&mut self) {
        while self.evaluate_next() {}
    }
    pub fn debug(&mut self) {
        println!(
            "\nAccumulator: {}, Program counter: {}, Current instruction:{:?}, Negative flag: {}",
            self.accumulator,
            self.program_counter,
            OpCode::try_from(self.get_addresses(self.program_counter)),
            self.negative_flag
        );
        while self.evaluate_next() {
            // println!("{:?}",self.mailbox);
            println!(
                "\nAccumulator: {}, Program counter: {}, Current instruction:{:?}, Negative flag: {}",
                self.accumulator,
                self.program_counter,
                OpCode::try_from(self.get_addresses(self.program_counter)),
                self.negative_flag
            );
        }
    }
    pub fn get_mailbox(&self) -> &Mailbox {
        &self.mailbox
    }
    pub fn get_program_counter(&self) -> &u16 {
        &self.program_counter
    }
    pub fn get_accumulator(&self) -> &u16 {
        &self.accumulator
    }
    pub fn get_current_instruction(&self) -> (Option<OpCode>, u16) {
        let literal = self.get_addresses(self.program_counter);
        let current_instruction = OpCode::try_from(literal);
        if let Ok(current_instruction) = current_instruction {
            (Some(current_instruction), literal)
        } else {
            (None, literal)
        }
    }
}
