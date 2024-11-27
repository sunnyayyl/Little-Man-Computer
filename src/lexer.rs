use crate::error::AssemblerError;
use crate::error::AssemblerError::{EndOfLineExpected, UnexpectedInstruction};
use crate::MemonicType;
use std::collections::HashMap;
use std::io::BufRead;
pub type LabelLookup = HashMap<String, u16>;
pub type LexerLineStructure = [Option<LineStructure>; 100];
pub type LexerResult = Result<LexerLineStructure, AssemblerError>;
#[derive(Debug, PartialEq)]
pub enum RightField {
    Literal(u16),
    Label(String),
}

#[derive(Debug)]
pub struct ErrorInfo {
    pub start: usize,
    pub end: usize,
    pub line: u16,
    pub literal: String,
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
    pub literal: String,
    pub line: u16,
}

impl LineStructure {
    fn new(line: u16, literal: String) -> Self {
        Self {
            left: None,
            instruction: None,
            right: None,
            literal,
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

pub struct Lexer {
    label_lookup: LabelLookup,
}
impl Lexer {
    pub fn new() -> Self {
        Lexer {
            label_lookup: Default::default(),
        }
    }
    pub fn parse<T: BufRead>(mut self, source: T) -> (LexerResult, LabelLookup) {
        let mut line: [Option<LineStructure>; 100] = [const { None }; 100];
        let mut current_line = 0;
        for (file_line, v) in source.lines().enumerate() {
            let mut expect = TokenType::Any;
            if let Ok(line_literal) = v {
                let mut current = LineStructure::new(file_line as u16, line_literal.clone());
                if line_literal.trim().starts_with("//") {
                    continue;
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
                            return (
                                Err(UnexpectedInstruction(
                                    ErrorInfo {
                                        start,
                                        end,
                                        line: file_line as u16,
                                        literal: line_literal.clone(),
                                    },
                                    instruction,
                                )),
                                self.label_lookup,
                            );
                        }
                    } else if expect == TokenType::LeftLabel || expect == TokenType::Any {
                        expect = TokenType::Instruction;
                        self.label_lookup
                            .insert(substring.to_string(), current_line as u16);
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
                        return (
                            Err(EndOfLineExpected(ErrorInfo {
                                start,
                                end,
                                line: file_line as u16,
                                literal: line_literal.clone(),
                            })),
                            self.label_lookup,
                        );
                    } else {
                        return (
                            Err(AssemblerError::InvalidInstruction(
                                ErrorInfo {
                                    start,
                                    end,
                                    line: file_line as u16,
                                    literal: line_literal.clone(),
                                },
                                substring.to_string(),
                            )),
                            self.label_lookup,
                        );
                    }
                }
                line[current_line] = Some(current);
                current_line += 1;
            } else {
                panic!("Failed to read line");
            }
        }
        (Ok(line), self.label_lookup)
    }
}
