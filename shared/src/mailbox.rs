#[cfg(feature = "std")]
use {
    bytemuck::checked::cast_slice,
    std::fs::File,
    std::io::{Read, Write},
    std::ops::{Index, IndexMut},
    std::vec::Vec,
};

use crate::opcodes::{AddressType, MemonicType, OpCode};
use bytemuck::checked::{try_cast_slice, CheckedCastError};
#[cfg(not(feature = "std"))]
use core::ops::{Index, IndexMut};

#[derive(Debug)]
pub enum MailboxError {
    #[cfg(feature = "std")]
    Io(std::io::Error),
    Cast(CheckedCastError),
}

#[derive(Debug)]
pub struct Mailbox([u16; 100]);
impl From<[u16; 100]> for Mailbox {
    fn from(s: [u16; 100]) -> Self {
        Self(s)
    }
}
impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Mailbox {
    pub fn new() -> Self {
        Self([0; 100])
    }
    pub fn set_instruction(
        &mut self,
        index: u16,
        p0: MemonicType,
        p1: Option<u16>,
        p2: AddressType,
    ) {
        self[index] = OpCode::from_mnemonic_type(p0, p1, p2).to_numeric_representation();
    }
    pub fn read_from_u8_slice(slice: &[u8]) -> Result<Self, MailboxError> {
        let new_slice = try_cast_slice::<u8, u16>(slice);
        match new_slice {
            Ok(new_slice) => {
                let mut s: [u16; 100] = [0; 100];
                s.copy_from_slice(new_slice);
                Ok(Self(s))
            }
            Err(e) => Err(MailboxError::Cast(e)),
        }
    }
    #[cfg(feature = "std")]
    pub fn export_to_file(&self, file: &mut File) -> Result<(), MailboxError> {
        match file.write_all(cast_slice::<u16, u8>(self.0.as_slice())) {
            Ok(_) => Ok(()),
            Err(e) => Err(MailboxError::Io(e)),
        }
    }
    #[cfg(feature = "std")]
    pub fn read_from_file(file: &mut File) -> Result<Self, MailboxError> {
        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Ok(_) => Mailbox::read_from_u8_slice(buffer.as_slice()),
            Err(e) => Err(MailboxError::Io(e)),
        }
    }
}

impl Index<usize> for Mailbox {
    type Output = u16;
    fn index(&self, index: usize) -> &u16 {
        if !(0..=99).contains(&index) {
            panic!("There are only 100 mailbox (0-99) addresses available, got {}",index)
        }
        unsafe { self.0.get_unchecked(index) } // Safe because we checked the bounds
    }
}

impl IndexMut<usize> for Mailbox {
    fn index_mut(&mut self, index: usize) -> &mut u16 {
        if !(0..=99).contains(&index) {
            panic!("There are only 100 mailbox (0-99) addresses available, got {}",index)
        }
        unsafe { self.0.get_unchecked_mut(index) } // Safe because we checked the bounds
    }
}

impl Index<u16> for Mailbox {
    type Output = u16;
    fn index(&self, index: u16) -> &u16 {
        &self[index as usize]
    }
}

impl IndexMut<u16> for Mailbox {
    fn index_mut(&mut self, index: u16) -> &mut u16 {
        &mut self[index as usize]
    }
}

#[cfg(feature = "std")]
impl From<Vec<u16>> for Mailbox {
    fn from(vec: Vec<u16>) -> Self {
        let mut s: [u16; 100] = [0; 100];
        for (i, v) in vec.iter().enumerate() {
            s[i] = *v;
        }
        Self(s)
    }
}
