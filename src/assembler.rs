use crate::error::AssemblerError;
use crate::error::AssemblerError::{
    InstructionExpected, InstructionExpectedAddress, InstructionExpectedGotLabels, UnsetLabel,
};
use crate::lexer::{ErrorInfo, LexerLineStructure, LineStructure, RightField};
use crate::OpCode;
use std::collections::HashMap;

pub enum State<T, E> {
    Ok(T),
    Err(E),
    Done,
}

pub struct Assembler {
    line_structure: LexerLineStructure,
    table_lookup: HashMap<String, u16>,
    current_line: u16,
}
impl<'a> Assembler {
    pub fn new(line_structure: LexerLineStructure, table_lookup: HashMap<String, u16>) -> Self {
        Self {
            line_structure,
            table_lookup,
            current_line: 0,
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
                    literal,
                    line,
                } => {
                    let address: u16;
                    match &right.value {
                        RightField::Literal(value) => {
                            address = *value;
                        }
                        RightField::Label(label) => {
                            if let Some(addr) = self.table_lookup.get(label) {
                                address = *addr;
                            } else {
                                return State::Err(UnsetLabel(
                                    ErrorInfo {
                                        start: right.start,
                                        end: right.end,
                                        line: *line,
                                        literal: literal.clone(),
                                    },
                                    label.clone(),
                                ));
                            }
                        }
                    }
                    if let Ok(instruction) =
                        OpCode::try_from_mnemonic_type(instruction.value, Some(address))
                    {
                        State::Ok(instruction)
                    } else {
                        State::Err(InstructionExpectedAddress(
                            ErrorInfo {
                                start: right.start,
                                end: right.end,
                                line: *line,
                                literal: literal.clone(),
                            },
                            instruction.value,
                        ))
                    }
                }
                LineStructure {
                    left,
                    instruction: Some(instruction),
                    right: None,
                    literal,
                    line,
                } => {
                    if let Some(left) = left {
                        self.table_lookup
                            .insert(left.value.to_string(), self.current_line);
                    }
                    if let Ok(instruction) = OpCode::try_from_mnemonic_type(instruction.value, None)
                    {
                        State::Ok(instruction)
                    } else {
                        State::Err(InstructionExpectedAddress(
                            ErrorInfo {
                                start: instruction.start,
                                end: instruction.end,
                                literal: literal.clone(),
                                line: *line,
                            },
                            instruction.value,
                        ))
                    }
                }
                LineStructure {
                    left: Some(left),
                    instruction: None,
                    right: None,
                    literal,
                    line,
                } => State::Err(InstructionExpectedGotLabels(ErrorInfo {
                    start: left.start,
                    end: left.end,
                    literal: literal.clone(),
                    line: *line,
                })),
                LineStructure {
                    left: None,
                    instruction: None,
                    right: Some(right),
                    literal,
                    line,
                } => State::Err(InstructionExpectedGotLabels(ErrorInfo {
                    start: right.start,
                    end: right.end,
                    literal: literal.clone(),
                    line: *line,
                })),
                LineStructure {
                    left: Some(left),
                    instruction: None,
                    right: Some(right),
                    literal,
                    line,
                } => State::Err(InstructionExpected(ErrorInfo {
                    start: left.start,
                    end: right.end,
                    literal: literal.clone(),
                    line: *line,
                })),
                LineStructure {
                    left: None,
                    instruction: None,
                    right: None,
                    literal,
                    line,
                } => {
                    panic!(
                        "Unexpected empty line structure at line {}, literal: {}",
                        line, literal
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
