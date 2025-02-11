#![no_std]
#[cfg(feature = "std")]
extern crate std;
mod mailbox;
pub use mailbox::Mailbox;
mod opcodes;
pub use opcodes::MemonicType;
pub use opcodes::OpCode;
#[cfg(feature = "std")]
mod std_runtime;
#[cfg(feature = "std")]
pub use std_runtime::StdRuntime;
pub mod runtime;
#[cfg(feature = "assembler")]
pub mod error;
#[cfg(feature = "assembler")]
pub mod lexer;
#[cfg(feature = "assembler")]
pub mod assembler;
#[cfg(feature = "std")]
mod stacked_std_runtime;
#[cfg(feature = "std")]
pub use stacked_std_runtime::StackedStdRuntime;