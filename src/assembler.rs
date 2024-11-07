use crate::assembler::AssemblerError::{EndOfLineExpected, InstructionExpected, UnsetLabel};
use crate::{MemonicType, OpCode};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io::BufRead;
use std::marker::PhantomData;
pub enum State<T, E> {
    Ok(T),
    Err(E),
    Done,
}
#[derive(Debug, PartialEq)]
pub enum RightField {
    Literal(u16),
    Label(String),
}
#[derive(Debug, PartialEq)]
pub enum TokenType {
    Any,
    LeftLabel,
    RightLabel,
    Instruction,
    Comment,
}
#[derive(Debug)]
pub enum AssemblerError {
    InstructionExpected(u16),
    EndOfLineExpected(u16),
    UnsetLabel(u16, String),
    InstructionUsedAsLabel(u16, String),
}
#[derive(Debug)]
pub struct LineStructure {
    pub left: Option<String>,
    pub instruction: Option<MemonicType>,
    pub right: Option<RightField>,
    pub line: u16,
}
impl LineStructure {
    fn new(line: u16) -> Self {
        Self {
            left: None,
            instruction: None,
            right: None,
            line,
        }
    }
}
pub struct Lexer<T>
where
    T: BufRead,
{
    label_lookup: HashMap<String, u16>,
    phantom: PhantomData<T>,
}

impl<T> Lexer<T>
where
    T: BufRead,
{
    pub fn new() -> Self {
        Lexer {
            label_lookup: Default::default(),
            phantom: Default::default(),
        }
    }
    pub fn parse(
        mut self,
        source: T,
    ) -> (
        Result<[Option<LineStructure>; 100], AssemblerError>,
        HashMap<String, u16>,
    ) {
        let mut line: [Option<LineStructure>; 100] = [const { None }; 100];
        let mut current_line = 0;
        for (file_line, v) in source.lines().enumerate() {
            let mut current = LineStructure::new(file_line as u16);
            let mut expect = TokenType::Any;
            if let Ok(line_literal) = v {
                if line_literal.starts_with("//") {
                    current_line -= 1; // cancel out the addition later on, skip comment-only line
                    continue;
                }
                for v in line_literal.split_whitespace() {
                    if v.starts_with("//") {
                        break;
                    }
                    if let Some(instruction) = MemonicType::from_string(v) {
                        if expect == TokenType::Instruction || expect == TokenType::Any {
                            expect = TokenType::RightLabel;
                            current.instruction = Some(instruction);
                        } else {
                            return (
                                Err(InstructionExpected(file_line as u16)),
                                self.label_lookup,
                            );
                        }
                    } else if expect == TokenType::LeftLabel || expect == TokenType::Any {
                        expect = TokenType::Instruction;
                        self.label_lookup.insert(v.to_string(), current_line as u16);
                        current.left = Some(v.to_string());
                    } else if expect == TokenType::RightLabel {
                        expect = TokenType::Comment;
                        if let Ok(number) = v.parse::<u16>() {
                            current.right = Some(RightField::Literal(number));
                        } else {
                            current.right = Some(RightField::Label(v.to_string()));
                        }
                    } else if expect == TokenType::Comment {
                        return (Err(EndOfLineExpected(file_line as u16)), self.label_lookup);
                    }
                }
            }
            line[current_line] = Some(current);
            current_line += 1;
        }
        (Ok(line), self.label_lookup)
    }
}

pub struct Assembler {
    line_structure: [Option<LineStructure>; 100],
    table_lookup: HashMap<String, u16>,
    current_line: u16,
}
impl Assembler {
    pub fn new(
        line_structure: [Option<LineStructure>; 100],
        table_lookup: HashMap<String, u16>,
    ) -> Self {
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
                    line,
                } => {
                    //if let Some(left) = left {
                    //    self.table_lookup.insert(left.to_string(), self.current_line);
                    //}
                    match right {
                        RightField::Literal(value) => {
                            State::Ok(OpCode::from_mnemonic_type(*instruction, Some(*value)))
                        }
                        RightField::Label(label) => {
                            if let Some(addr) = self.table_lookup.get(label) {
                                State::Ok(OpCode::from_mnemonic_type(*instruction, Some(*addr)))
                            } else {
                                State::Err(UnsetLabel(*line, label.clone()))
                            }
                        }
                    }
                }
                LineStructure {
                    left: Some(left),
                    instruction: Some(instruction),
                    right: None,
                    ..
                } => {
                    self.table_lookup
                        .insert(left.to_string(), self.current_line);
                    State::Ok(OpCode::from_mnemonic_type(*instruction, None))
                }
                LineStructure {
                    left: Some(_),
                    instruction: _,
                    right: _,
                    line,
                } => State::Err(InstructionExpected(*line)),
                LineStructure {
                    left: None,
                    instruction: Some(instruction),
                    right: None,
                    ..
                } => State::Ok(OpCode::from_mnemonic_type(*instruction, None)),

                LineStructure { line, .. } => State::Err(InstructionExpected(*line)),
            }
        } else {
            State::Done
        }
    }
    pub fn current_line(&self) -> u16 {
        self.current_line
    }
}
