use crate::error::AssemblerError::{EndOfLineExpected, UnexpectedInstruction};
use crate::error::{AssemblerError, ErrorInfo};
use crate::MemonicType;
use std::collections::HashMap;
use std::io::{BufRead, Lines};
use std::iter::Enumerate;

pub type LabelLookup = HashMap<String, u16>;
pub type LexerResult = [Option<LineStructure>; 100];
#[derive(Debug, PartialEq)]
pub enum RightField {
    Literal(u16),
    Label(String),
}
#[derive(Debug)]
pub enum LexerState {
    Some(LineStructure),
    Err(AssemblerError),
    Skip,
}
impl<V: FromIterator<Option<LineStructure>>> FromIterator<LexerState>
    for Result<V, AssemblerError>
{
    fn from_iter<T: IntoIterator<Item = LexerState>>(iter: T) -> Result<V, AssemblerError> {
        iter.into_iter()
            .map(|i| match i {
                LexerState::Some(v) => Ok(Some(v)),
                LexerState::Err(e) => Err(e),
                LexerState::Skip => Ok(None),
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct LinePart<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
}

#[derive(Debug)]
pub struct LineStructure {
    pub left: Option<LinePart<String>>,
    pub instruction: Option<LinePart<MemonicType>>,
    pub right: Option<LinePart<RightField>>,
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

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Any,
    LeftLabel,
    RightLabel,
    Instruction,
    Eof,
}

fn split_whitespace_with_index(s: &str) -> impl Iterator<Item = (&str, usize)> {
    s.split_whitespace()
        .map(move |sub| (sub, sub.as_ptr() as usize - s.as_ptr() as usize))
}

pub struct Lexer<T: BufRead> {
    label_lookup: LabelLookup,
    lines: Enumerate<Lines<T>>,
    assembly_line: usize,
}
impl<T: BufRead> Lexer<T> {
    pub fn new(line: Lines<T>) -> Self {
        Lexer {
            label_lookup: Default::default(),
            lines: line.enumerate(),
            assembly_line: 0,
        }
    }
    pub fn get_label_lookup(&self) -> &LabelLookup {
        &self.label_lookup
    }
}
impl<T: BufRead> Iterator for Lexer<T> {
    type Item = LexerState;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some((file_line, line_literal)) => {
                let mut expect = TokenType::Any;
                if let Ok(line_literal) = line_literal {
                    let mut current = LineStructure::new(file_line as u16);
                    if line_literal.trim().starts_with("//") {
                        return Some(LexerState::Skip);
                    }
                    for (substring, index) in split_whitespace_with_index(&line_literal) {
                        if substring.starts_with("//") {
                            break;
                        }
                        let start = index;
                        let end = index + substring.len();
                        if let Some(instruction) = MemonicType::from_string(substring) {
                            if expect == TokenType::Instruction || expect == TokenType::Any {
                                expect = TokenType::RightLabel;
                                current.instruction = Some(LinePart {
                                    start,
                                    end,
                                    value: instruction,
                                });
                            } else {
                                return Some(LexerState::Err(UnexpectedInstruction(
                                    ErrorInfo {
                                        start,
                                        end,
                                        line: file_line as u16,
                                        literal: line_literal.clone(),
                                    },
                                    instruction,
                                )));
                            }
                        } else if expect == TokenType::LeftLabel || expect == TokenType::Any {
                            expect = TokenType::Instruction;
                            self.label_lookup
                                .insert(substring.to_string(), self.assembly_line as u16);
                            current.left = Some(LinePart {
                                start,
                                end,
                                value: substring.to_string(),
                            });
                        } else if expect == TokenType::RightLabel {
                            expect = TokenType::Eof;
                            if let Ok(number) = substring.parse::<u16>() {
                                current.right = Some(LinePart {
                                    start,
                                    end,
                                    value: RightField::Literal(number),
                                });
                            } else {
                                current.right = Some(LinePart {
                                    start,
                                    end,
                                    value: RightField::Label(substring.to_string()),
                                });
                            }
                        } else if expect == TokenType::Eof {
                            return Some(LexerState::Err(EndOfLineExpected(ErrorInfo {
                                start,
                                end,
                                line: file_line as u16,
                                literal: line_literal.clone(),
                            })));
                        } else {
                            return Some(LexerState::Err(AssemblerError::InvalidInstruction(
                                ErrorInfo {
                                    start,
                                    end,
                                    line: file_line as u16,
                                    literal: line_literal.clone(),
                                },
                                substring.to_string(),
                            )));
                        }
                    }
                    self.assembly_line += 1;
                    Some(LexerState::Some(current))
                } else {
                    panic!("Failed to read line");
                }
            }
            None => None,
        }
    }
}
