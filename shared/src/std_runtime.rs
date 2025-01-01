use crate::mailbox::Mailbox;
use crate::runtime::{Runtime, RuntimeCommon, RuntimeState};
use std::io::{stdin, BufRead};

pub struct StdRuntime {
    pub common: RuntimeCommon,
}

impl StdRuntime {
    pub fn new(p0: Mailbox) -> Self {
        Self {
            common: RuntimeCommon {
                accumulator: 0,
                program_counter: 0,
                negative_flag: false,
                mailbox: p0,
            },
        }
    }
}

impl Runtime for StdRuntime {
    fn get_common(&self) -> &RuntimeCommon {
        &self.common
    }
    fn get_common_mut(&mut self) -> &mut RuntimeCommon {
        &mut self.common
    }
    fn inp(&mut self, _: Option<u16>) -> RuntimeState {
        let mut line = String::new();
        {
            let mut lock = stdin().lock();
            lock.read_line(&mut line).unwrap();
        }
        self.common.accumulator = line.trim().parse::<u16>().expect("Input must be a number");
        RuntimeState::Running
    }
    fn out(&mut self, _: Option<u16>) -> RuntimeState {
        println!("{}", self.common.accumulator);
        RuntimeState::Running
    }

    fn sout(&mut self, _: Option<u16>) -> RuntimeState {
        let char =
            u8::try_from(self.common.accumulator).expect("Cannot be converted to ascii character") as char;
        print!("{}", char);
        RuntimeState::Running
    }
}
