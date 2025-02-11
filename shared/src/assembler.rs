use crate::error::AssemblerError::{
    InstructionExpected, InstructionExpectedAddress, InstructionExpectedGotLabels, UnsetLabel,
};
use crate::error::{AssemblerError, ErrorInfo};
use crate::lexer::{LexerResult, LineStructure, RightField};
use crate::opcodes::AddressType;
use crate::OpCode;
use std::collections::HashMap;
use std::io::{BufRead, Lines};
use std::string::{String, ToString};

pub enum State<T, E> {
    Ok(T),
    Err(E),
    Done,
}

pub struct Assembler<T: BufRead> {
    line_structure: LexerResult,
    table_lookup: HashMap<String, u16>,
    current_line: u16,
    source: Lines<T>,
}
impl<'a, T: BufRead> Assembler<T> {
    pub fn new(
        source: Lines<T>,
        table_lookup: HashMap<String, u16>,
        line_structure: LexerResult,
    ) -> Self {
        Self {
            line_structure,
            table_lookup,
            current_line: 0,
            source,
        }
    }
    pub fn parse_line(&mut self) -> State<OpCode, AssemblerError> {
        let current_line = &self.line_structure[self.current_line as usize];
        self.current_line += 1;
        if let Some(line) = current_line {
            match line {
                LineStructure {
                    left: _left,
                    instruction: Some(instruction),
                    right: Some(right),
                    line,
                } => {
                    let address: u16;
                    let mut address_type: AddressType;
                    match &right.value {
                        RightField::Literal(value) => {
                            address = *value;
                            address_type = AddressType::Literal;
                        }
                        RightField::Label(label) => {
                            if let Some(addr) = self.table_lookup.get(label) {
                                address = *addr;
                            } else {
                                return State::Err(UnsetLabel(
                                    ErrorInfo::new(right.start, right.end, *line, &mut self.source),
                                    label.clone(),
                                ));
                            }
                            address_type = AddressType::Literal;
                        }
                        RightField::Register(name) => {
                            let n = name.to_uppercase();
                            if n == "SP" {
                                address = 99;
                            }else if n=="ACCUMULATOR"{
                                address = 98;
                            }  else {
                                panic!("Invalid register name: {} at line {}", n, *line);
                            }
                            address_type = AddressType::Literal;
                        }
                        RightField::LiteralPointer(value) => {
                            address = *value;
                            address_type = AddressType::Pointer;
                        }
                        RightField::LabelPointer(label) => {
                            if let Some(addr) = self.table_lookup.get(label) {
                                address = *addr;
                            } else {
                                return State::Err(UnsetLabel(
                                    ErrorInfo::new(right.start, right.end, *line, &mut self.source),
                                    label.clone(),
                                ));
                            }
                            address_type = AddressType::Pointer;
                        }
                        RightField::RegisterPointer(name) => {
                            let n = name.to_uppercase();
                            if n == "SP" {
                                address = 99;
                            } else if n=="ACCUMULATOR"{
                                address = 98;
                            } else {
                                panic!("Invalid register name: {} at line {}", n, *line);
                            }
                            address_type = AddressType::Pointer;
                        }
                    }
                    if let Ok(instruction) = OpCode::try_from_mnemonic_type(
                        instruction.value,
                        Some(address),
                        address_type,
                    ) {
                        State::Ok(instruction)
                    } else {
                        State::Err(InstructionExpectedAddress(
                            ErrorInfo::new(right.start, right.end, *line, &mut self.source),
                            instruction.value,
                        ))
                    }
                }
                #[allow(unused_variables)]
                LineStructure {
                    left,
                    instruction: Some(instruction),
                    right: None,
                    line,
                } => {
                    if let Some(left) = left {
                        self.table_lookup
                            .insert(left.value.to_string(), self.current_line);
                    }
                    if let Ok(instruction) = OpCode::try_from_mnemonic_type(
                        instruction.value,
                        None,
                        AddressType::Literal,
                    ) {
                        State::Ok(instruction)
                    } else {
                        State::Err(InstructionExpectedAddress(
                            ErrorInfo::new(
                                instruction.start,
                                instruction.end,
                                *line,
                                &mut self.source,
                            ),
                            instruction.value,
                        ))
                    }
                }
                #[allow(unused_variables)]
                LineStructure {
                    left: Some(left),
                    instruction: None,
                    right: None,
                    line,
                } => State::Err(InstructionExpectedGotLabels(ErrorInfo::new(
                    left.start,
                    left.end,
                    *line,
                    &mut self.source,
                ))),
                #[allow(unused_variables)]
                LineStructure {
                    left: None,
                    instruction: None,
                    right: Some(right),
                    line,
                } => State::Err(InstructionExpectedGotLabels(ErrorInfo::new(
                    right.start,
                    right.end,
                    *line,
                    &mut self.source,
                ))),
                #[allow(unused_variables)]
                LineStructure {
                    left: Some(left),
                    instruction: None,
                    right: Some(right),
                    line,
                } => State::Err(InstructionExpected(ErrorInfo::new(
                    left.start,
                    right.end,
                    *line,
                    &mut self.source,
                ))),
                #[allow(unused_variables)]
                LineStructure {
                    left: None,
                    instruction: None,
                    right: None,
                    line,
                } => {
                    panic!(
                        "Unexpected empty line structure at line {}, literal: {}",
                        line,
                        self.source.nth(*line as usize).unwrap().unwrap()
                    )
                }
            }
        } else {
            State::Done
        }
    }
    pub fn current_line(&self) -> u16 {
        self.current_line
    }
}
