#![cfg_attr(not(feature="std"), no_std)]
extern crate alloc;

pub mod opcodes;
pub mod std_runtime;
pub mod mailbox;
pub mod runtime;
