use crate::MemonicType;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, Lines};

#[derive(Debug)]
pub struct ErrorInfo {
    pub start: usize,
    pub end: usize,
    pub line: u16,
    pub literal: String,
}
impl ErrorInfo {
    pub fn new<T: BufRead>(start: usize, end: usize, line: u16, source: &mut Lines<T>) -> Self {
        Self {
            start,
            end,
            line,
            literal: source.nth(line as usize).unwrap().unwrap()[start..end].to_string(),
        }
    }
}
fn show_code_and_point_at_position(
    f: &mut Formatter<'_>,
    position: &ErrorInfo,
) -> std::fmt::Result {
    writeln!(f, "{}", position.literal)?;
    write!(
        f,
        "{}",
        " ".repeat(position.start) + &*"^".repeat(position.end - position.start)
    )
}
#[derive(Debug)]
pub enum AssemblerError {
    InstructionExpected(ErrorInfo),
    InstructionExpectedGotLabels(ErrorInfo),
    EndOfLineExpected(ErrorInfo),
    UnsetLabel(ErrorInfo, String),
    UnexpectedInstruction(ErrorInfo, MemonicType),
    InstructionExpectedAddress(ErrorInfo, MemonicType),
    InvalidInstruction(ErrorInfo, String),
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::InstructionExpected(info) => {
                writeln!(f, "Instruction expected at line {}", info.line + 1)?;
                show_code_and_point_at_position(f, info)
            }
            AssemblerError::InstructionExpectedGotLabels(info) => {
                writeln!(
                    f,
                    "Instruction expected at line {}, got labels",
                    info.line + 1
                )?;
                show_code_and_point_at_position(f, info)
            }
            AssemblerError::EndOfLineExpected(info) => {
                writeln!(f, "End of line expected at line {}", info.line + 1)?;
                show_code_and_point_at_position(f, info)
            }
            AssemblerError::UnsetLabel(info, label) => {
                writeln!(f, "Unset label {} at line {}", label, info.line + 1)?;
                show_code_and_point_at_position(f, info)
            }
            AssemblerError::UnexpectedInstruction(info, label) => {
                writeln!(
                    f,
                    "Unexpected instruction: {} at line {}",
                    label,
                    info.line + 1
                )?;
                show_code_and_point_at_position(f, info)
            }
            AssemblerError::InstructionExpectedAddress(info, instruction) => {
                writeln!(
                    f,
                    "Instruction {} expects an address at line {}",
                    instruction,
                    info.line + 1
                )?;
                show_code_and_point_at_position(f, info)
            }
            AssemblerError::InvalidInstruction(info, instruction) => {
                writeln!(
                    f,
                    "Invalid instruction {} at line {}",
                    instruction,
                    info.line + 1
                )?;
                show_code_and_point_at_position(f, info)
            }
        }
    }
}
