use crate::assembler::ParserError::{
    EndOfLineExpected, InstructionExpected, InstructionUsedAsLabel,
};
use crate::{Mailbox, MemonicType, OpCode};
use std::collections::HashMap;
macro_rules! set_instruction {
    ($self:ident,$mailbox:ident,$instruction:ident,$third:ident) => {
        match $self.set_instruction(&mut $mailbox, $instruction, $third) {
            State::Ok(_) => {}
            State::Err(e) => return Err(e),
        }
    };
}
enum State<T, E> {
    Ok(T),
    Err(E),
}
#[derive(Debug)]
pub enum ParserError {
    InstructionExpected(u16),
    EndOfLineExpected(u16),
    UnsetLabel(u16, String),
    InstructionUsedAsLabel(u16, String),
}
pub struct Parser {
    lines: Vec<String>,
    current_line: u16,
    label_lookup: HashMap<String, u16>,
}

impl Parser {
    pub fn new(buffer: Vec<String>) -> Self {
        Self {
            lines: buffer,
            current_line: 0,
            label_lookup: Default::default(),
        }
    }
    fn set_instruction(
        &self,
        mailbox: &mut Mailbox,
        instruction: MemonicType,
        label: Option<&str>,
    ) -> State<(), ParserError> {
        match label {
            Some(label) => {
                if let Ok(value) = label.parse() {
                    mailbox.set(
                        self.current_line,
                        OpCode::from_mnemonic_type(instruction, Some(value))
                            .to_numeric_representation(),
                    );
                    State::Ok(())
                } else {
                    let addr = self.label_lookup.get(label);
                    match addr {
                        Some(address) => {
                            mailbox.set_instruction(self.current_line, instruction, Some(*address));
                            State::Ok(())
                        }
                        None => State::Err(ParserError::UnsetLabel(
                            self.current_line,
                            label.to_string(),
                        )),
                    }
                }
            }
            None => {
                mailbox.set_instruction(self.current_line, instruction, None);
                State::Ok(())
            }
        }
    }
    pub fn parse(&mut self) -> Result<Mailbox, ParserError> {
        self.current_line = (self.lines.len() - 1) as u16;
        for line in self.lines.iter().rev() {
            let mut words = line.split_whitespace();
            let first_word = words.next();
            let second_word = words.next();
            let third_word = words.next();
            let forth_word = words.next();
            match (first_word, second_word, third_word, forth_word) {
                (None, None, None, None) => {
                    //blank line
                    break;
                }
                (Some(first_word), Some(_), _, _) => {
                    if MemonicType::from_string(first_word).is_none() {
                        self.label_lookup
                            .insert(first_word.to_string(), self.current_line);
                    }
                }
                (Some(instruction), None, None, None) => {
                    assert!(MemonicType::from_string(instruction).is_some());
                }
                (_, _, _, _) => todo!(),
            }
            if self.current_line > 0 {
                self.current_line -= 1;
            }
        }
        println!("{:?}", self.label_lookup);
        self.current_line = 0;
        let mut mailbox = Mailbox([0_u16; 100]);
        for line in &self.lines {
            let mut words = line.split_whitespace();
            let first_word = words.next();
            let second_word = words.next();
            let third_word = words.next();
            let forth_word = words.next();
            match (first_word, second_word, third_word, forth_word) {
                (None, None, None, None) => {
                    //blank line
                    break;
                }
                (Some(first_word), None, None, None) => {
                    // Instruction only
                    if let Some(instruction) = MemonicType::from_string(first_word) {
                        let l: Option<&str> = None;
                        set_instruction!(self, mailbox, instruction, l)
                    } else {
                        return Err(InstructionExpected(self.current_line));
                    }
                }
                (Some(first_word), Some(second_word), None, None) => {
                    if let Some(instruction) = MemonicType::from_string(first_word) {
                        // Instruction Right
                        let l: Option<&str> = Some(second_word);
                        set_instruction!(self, mailbox, instruction, l)
                    } else if let Some(instruction) = MemonicType::from_string(second_word) {
                        // Left Instruction
                        self.label_lookup
                            .insert(first_word.to_string(), self.current_line);
                        let l: Option<&str> = None;
                        set_instruction!(self, mailbox, instruction, l)
                    } else {
                        return Err(InstructionExpected(self.current_line));
                    }
                }
                (Some(first_word), Some(second_word), third_word, forth_word) => {
                    if third_word.unwrap_or("//").starts_with("//") {
                        if let Some(instruction) = MemonicType::from_string(first_word) {
                            // Instruction Right //comment
                            let l: Option<&str> = Some(second_word);
                            set_instruction!(self, mailbox, instruction, l)
                        } else if let Some(instruction) = MemonicType::from_string(second_word) {
                            // Left Instruction //comment
                            self.label_lookup
                                .insert(first_word.to_string(), self.current_line);
                            let l: Option<&str> = None;
                            set_instruction!(self, mailbox, instruction, l)
                        }
                    } else if let Some(third_word) = third_word {
                        // Left Instruction Right //Comment
                        if MemonicType::from_string(first_word).is_some() {
                            // Label cannot have the same name as instruction
                            return Err(InstructionUsedAsLabel(
                                self.current_line,
                                first_word.to_string(),
                            ));
                        } else if MemonicType::from_string(third_word).is_some() {
                            return Err(InstructionUsedAsLabel(
                                self.current_line,
                                third_word.to_string(),
                            ));
                        }
                        if forth_word.unwrap_or("//").starts_with("//") {
                            // ignore comment
                        } else {
                            return Err(EndOfLineExpected(self.current_line));
                        }
                        if let Some(instruction) = MemonicType::from_string(second_word) {
                            self.label_lookup
                                .insert(first_word.to_string(), self.current_line);
                            let l: Option<&str> = Some(third_word);
                            set_instruction!(self, mailbox, instruction, l)
                        } else {
                            return Err(InstructionExpected(self.current_line));
                        }
                    } else {
                        todo!()
                    }
                }

                (_, _, _, _) => return Err(InstructionExpected(self.current_line)),
            }
            self.current_line += 1;
        }
        Ok(mailbox)
    }
}
