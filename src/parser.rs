use crate::{Mailbox, MemonicType, OpCode};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
#[derive(Debug)]
enum ParserError {
    InvalidInstruction(String),
    InstructionExpected,
}
struct Token {
    left: Option<String>,
    instruction: MemonicType,
    right: Option<String>,
}
pub(crate) struct Lexer<'a> {
    source: &'a [char],
    current_position: usize,
    current_character: char,
    label_lookup: HashMap<String, u16>,
    counter: u16,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(s: &'a [char]) -> Self {
        let lexer = Lexer {
            source: s,
            current_position: 0,
            current_character: s[0],
            label_lookup: HashMap::new(),
            counter: 0,
        };
        lexer
    }
    fn read_label(&mut self) -> Option<String> {
        let original_position = self.current_position;
        let s = self.read_string();
        match s {
            Some(s) => {
                if MemonicType::from_string(&s).is_none() {
                    Some(s)
                } else {
                    self.current_position = original_position;
                    self.current_character = self.source[original_position];
                    None
                }
            }
            None => None,
        }
    }
    fn read_string(&mut self) -> Option<String> {
        let original_position = self.current_position;
        let mut string = String::new();
        while self.current_character.is_alphabetic() {
            string.push(self.current_character);
            self.next();
        }
        if string.is_empty() {
            self.current_position = original_position;
            self.current_character = self.source[original_position];
            None
        } else {
            Some(string)
        }
    }
    fn next(&mut self) {
        self.current_position += 1;
        if self.current_position < self.source.len() {
            self.current_character = self.source[self.current_position];
        } else {
            self.current_character = '\0';
        }
    }
    fn read_instruction(&mut self) -> Result<MemonicType, ParserError> {
        let instruction = self.read_string();
        match instruction {
            Some(s) => match s.as_str() {
                "ADD" => Ok(MemonicType::ADD),
                "SUB" => Ok(MemonicType::SUB),
                "STA" => Ok(MemonicType::STA),
                "LDA" => Ok(MemonicType::LDA),
                "BRA" => Ok(MemonicType::BRA),
                "BRZ" => Ok(MemonicType::BRZ),
                "BRP" => Ok(MemonicType::BRP),
                "INP" => Ok(MemonicType::INP),
                "OUT" => Ok(MemonicType::OUT),
                "HLT" => Ok(MemonicType::HLT),
                "COB" => Ok(MemonicType::COB),
                _ => Err(ParserError::InvalidInstruction(s)),
            },
            None => Err(ParserError::InstructionExpected),
        }
    }
    fn read_line(&mut self) -> OpCode {
        let left = self.read_label();
        let instruction = self.read_instruction();
        let right = self.read_label();
        match instruction {
            Ok(i) => {
                if let Some(s) = left {
                    let entry = self.label_lookup.entry(s);
                    if let Entry::Vacant(e) = entry {
                        e.insert(self.counter);
                    } else {
                        entry.or_insert(self.counter); // Should I panic here instead? Can the label be overwritten?
                    }
                }
                if let Some(s) = right {
                    return OpCode::from_mnemonic_type(i, Some(self.label_lookup[&s]));
                }
                // compile time check for instruction that needs an address
                match i {
                    MemonicType::INP => OpCode::INP(None),
                    MemonicType::OUT => OpCode::OUT(None),
                    MemonicType::HLT => OpCode::HLT(None),
                    MemonicType::COB => OpCode::COB(None),
                    _ => panic!("{} requires an address", i),
                }
            }
            Err(e) => {
                panic!("Error reading instruction: {:?}", e);
            }
        }
    }
    fn consume_whitespace(&mut self) {
        while self.current_character.is_whitespace() {
            self.next();
        }
    }
    pub(crate) fn parse(&mut self) -> Mailbox {
        let mut mailbox = Mailbox([0; 100]);
        while self.current_character != '\0' {
            mailbox.set(self.counter, self.read_line().to_numeric_representation());
            self.counter += 1;
            assert!(
                self.current_character == '\n'
                    || self.current_character == '\0'
                    || self.current_character == '\r'
                    || self.current_character == '\n'
            );
            self.next();
            self.consume_whitespace();
        }
        mailbox
    }
}
