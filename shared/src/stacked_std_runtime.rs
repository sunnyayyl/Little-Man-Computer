use crate::mailbox::Mailbox;
use crate::runtime::{Runtime, RuntimeCommon, RuntimeState, Stack};
use std::io::{stdin, BufRead};
use std::string::String;
use std::{print, println};

//const HEAP_START: u16 = 49;
const STACK_START: u16 = 100 - 5;
//const STACK_CAPACITY: u16 = 25;
//const HEAP_CAPACITY: u16 = 24;

pub struct StackedStdRuntime {
    pub common: RuntimeCommon,
    pub stack_pointer: u16,
}

impl StackedStdRuntime {
    pub fn new(p0: Mailbox) -> Self {
        Self {
            common: RuntimeCommon {
                accumulator: 0,
                program_counter: 0,
                negative_flag: false,
                mailbox: p0,
            },
            stack_pointer: STACK_START,
        }
    }
}

impl Runtime for StackedStdRuntime {
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
        let char = u8::try_from(self.common.accumulator)
            .expect("Cannot be converted to ascii character") as char;
        print!("{}", char);
        RuntimeState::Running
    }

    fn pop(&mut self, _addr: Option<u16>) -> RuntimeState {
        self.common.accumulator = self.stack_pop();
        RuntimeState::Running
    }

    fn push(&mut self, _addr: Option<u16>) -> RuntimeState {
        self.stack_push(self.common.accumulator);
        RuntimeState::Running
    }
    fn load(&self, addr: u16) -> u16 {
        if addr==99{
            self.stack_pointer
        }else if addr==98{
            self.common.accumulator
        }else{
            self.get_common().mailbox[addr]
        }
    }
    fn store(&mut self, addr: u16, value: u16) {
        if addr==99{
            self.stack_pointer = value;
        }else if addr==98{
            self.common.accumulator = value;
        }else{
            self.get_common_mut().mailbox[addr] = value;
        }
    }
}
impl Stack for StackedStdRuntime {
    fn stack_push(&mut self, value: u16) {
        self.stack_pointer -= 1;
        self.common.mailbox[self.stack_pointer] = value;
    }
    fn stack_pop(&mut self) -> u16 {
        let value = self.common.mailbox[self.stack_pointer];
        self.stack_pointer += 1;
        value
    }
}
