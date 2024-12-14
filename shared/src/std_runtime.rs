use crate::create_getter;
use crate::mailbox::Mailbox;
use crate::runtime::{Runtime, RuntimeState};
use paste::paste;
#[cfg(feature = "std")]
use std::io::{stdin, BufRead};

pub struct StdRuntime {
    pub accumulator: u16,
    pub program_counter: u16,
    pub negative_flag: bool,
    pub mailbox: Mailbox,
}

impl StdRuntime {
    pub fn new(p0: Mailbox) -> Self {
        Self {
            accumulator: 0,
            program_counter: 0,
            negative_flag: false,
            mailbox: p0,
        }
    }
}

impl Runtime for StdRuntime {
    create_getter!(
        (accumulator, u16),
        (program_counter, u16),
        (negative_flag, bool),
        (mailbox, Mailbox)
    );
    fn inp(&mut self, _: Option<u16>) -> RuntimeState {
        let mut line = String::new();
        {
            let mut lock = stdin().lock();
            lock.read_line(&mut line).unwrap();
        }
        self.accumulator = line.trim().parse::<u16>().expect("Input must be a number");
        RuntimeState::Running
    }
    fn out(&mut self, _: Option<u16>) -> RuntimeState {
        println!("{}", self.accumulator);
        RuntimeState::Running
    }

    fn sout(&mut self, _: Option<u16>) -> RuntimeState {
        let char =
            u8::try_from(self.accumulator).expect("Cannot be converted to ascii character") as char;
        print!("{}", char);
        RuntimeState::Running
    }
}
