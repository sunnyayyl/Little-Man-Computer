#![cfg_attr(not(feature="std"), no_std)]
extern crate alloc;
mod mailbox;
pub use mailbox::Mailbox;
mod opcodes;
pub use opcodes::OpCode;
pub use opcodes::MemonicType;
#[cfg(feature = "std")]
mod std_runtime;
#[cfg(feature = "std")]
pub use std_runtime::StdRuntime;
pub mod runtime;
