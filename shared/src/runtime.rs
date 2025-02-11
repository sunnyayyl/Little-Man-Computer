use crate::mailbox::Mailbox;
use crate::opcodes::{AddressType, OpCode};
#[cfg(not(feature = "std"))]
use core::{
    option::{Option, Option::None, Option::Some},
    result::Result::Ok,
};
pub enum RuntimeError {
    InvalidInstruction(u16, u16),
}
pub enum RuntimeState {
    Running,
    Halted,
    Error(RuntimeError),
}

impl RuntimeState {
    pub fn is_running(&self) -> bool {
        matches!(self, RuntimeState::Running)
    }
    pub fn is_halted(&self) -> bool {
        matches!(self, RuntimeState::Halted)
    }
    pub fn is_error(&self) -> bool {
        matches!(self, RuntimeState::Error(_))
    }
}
#[allow(dead_code)]
fn wrap_between_valid_values(value: u16) -> u16 {
    if value > 999 {
        wrap_between_valid_values(value - 1000)
    } else {
        value
    }
}
pub trait Stack {
    fn stack_push(&mut self, value: u16);
    fn stack_pop(&mut self) -> u16;
}

pub struct RuntimeCommon {
    pub accumulator: u16,
    pub program_counter: u16,
    pub negative_flag: bool,
    pub mailbox: Mailbox,
}
pub trait Runtime {
    fn get_common(&self) -> &RuntimeCommon;
    fn get_common_mut(&mut self) -> &mut RuntimeCommon;
    fn add(&mut self, addr: Option<u16>) -> RuntimeState {
        let new_value = self.get_common().accumulator
            + self.load(addr.expect("ADD requires an address"));
        let common = self.get_common_mut();
        common.accumulator = wrap_between_valid_values(new_value);
        common.negative_flag = false;
        RuntimeState::Running
    }
    fn sub(&mut self, addr: Option<u16>) -> RuntimeState {
        let current_box = self.load(addr.expect("SUB requires an address"));
        let common = self.get_common_mut();
        if common.accumulator < current_box {
            common.negative_flag = true;
            common.accumulator = current_box - common.accumulator;
        } else {
            common.accumulator -= current_box;
        } // Should overflow result in a negative flag?
        RuntimeState::Running
    }
    fn mult(&mut self, addr: Option<u16>) -> RuntimeState {
        let new_value = self.get_common().accumulator
            * self.load(addr.expect("MULT requires an address"));
        let common = self.get_common_mut();
        common.accumulator = wrap_between_valid_values(new_value);
        common.negative_flag = false;
        RuntimeState::Running
    }
    fn load(&self, addr: u16) -> u16{
        self.get_common().mailbox[addr]
    }
    fn store(&mut self, addr: u16, value: u16){
        self.get_common_mut().mailbox[addr] = value;
    }
    fn sta(&mut self, addr: Option<u16>) -> RuntimeState {
        self.store(addr.expect("STA requires an address"), self.get_common().accumulator);
        RuntimeState::Running
    }
    fn lda(&mut self, addr: Option<u16>) -> RuntimeState {
        let v = self.load(addr.expect("LDA requires an address"));
        let common = self.get_common_mut();
        common.accumulator = v;
        RuntimeState::Running
    }
    fn bra(&mut self, addr: Option<u16>) -> RuntimeState {
        let common = self.get_common_mut();
        common.program_counter = addr.expect("BRA require an addresses");
        RuntimeState::Running
    }
    fn brz(&mut self, addr: Option<u16>) -> RuntimeState {
        let common = self.get_common_mut();
        if common.accumulator == 0 && !common.negative_flag {
            // Should the negative flag be taken into account?
            common.program_counter = addr.expect("BRZ require an addresses");
        }
        RuntimeState::Running
    }
    fn brp(&mut self, addr: Option<u16>) -> RuntimeState {
        let common = self.get_common_mut();
        if !common.negative_flag {
            common.program_counter = addr.expect("BRP require an addresses");
        }
        RuntimeState::Running
    }
    fn inp(&mut self, addr: Option<u16>) -> RuntimeState;
    fn out(&mut self, addr: Option<u16>) -> RuntimeState;
    fn sout(&mut self, addr: Option<u16>) -> RuntimeState;
    fn pop(&mut self, addr: Option<u16>) -> RuntimeState;
    fn push(&mut self, addr: Option<u16>) -> RuntimeState;
    fn evaluate_current(&mut self) -> RuntimeState {
        let current_instruction =
            OpCode::try_from(self.load(self.get_common().program_counter));
        if let Ok(current_instruction) = current_instruction {
            self.get_common_mut().program_counter += 1;
            match current_instruction {
                OpCode::ADD(addr, AddressType::Literal) => self.add(addr),
                OpCode::ADD(addr, AddressType::Pointer) => {
                    self.add(Some(self.load(addr.unwrap())))
                }
                OpCode::SUB(addr, AddressType::Literal) => self.sub(addr),
                OpCode::SUB(addr, AddressType::Pointer) => {
                    self.sub(Some(self.load(addr.unwrap())))
                }
                OpCode::MULT(addr, AddressType::Literal) => self.mult(addr),
                OpCode::MULT(addr, AddressType::Pointer) => {
                    self.mult(Some(self.load(addr.unwrap())))
                }
                OpCode::STA(addr, AddressType::Literal) => self.sta(addr),
                OpCode::STA(addr, AddressType::Pointer) => {
                    self.sta(Some(self.load(addr.unwrap())))
                }
                OpCode::LDA(addr, AddressType::Literal) => self.lda(addr),
                OpCode::LDA(addr, AddressType::Pointer) => {
                    self.lda(Some(self.load(addr.unwrap())))
                }
                OpCode::BRA(addr, AddressType::Literal) => self.bra(addr),
                OpCode::BRA(addr, AddressType::Pointer) => {
                    self.bra(Some(self.load(addr.unwrap())))
                }
                OpCode::BRZ(addr, AddressType::Literal) => self.brz(addr),
                OpCode::BRZ(addr, AddressType::Pointer) => {
                    self.brz(Some(self.load(addr.unwrap())))
                }
                OpCode::BRP(addr, AddressType::Literal) => self.brp(addr),
                OpCode::BRP(addr, AddressType::Pointer) => {
                    self.brp(Some(self.load(addr.unwrap())))
                }
                OpCode::OUT(addr, AddressType::Literal) => self.out(addr),
                OpCode::OUT(addr, AddressType::Pointer) => {
                    self.out(Some(self.load(addr.unwrap())))
                }
                OpCode::INP(addr, AddressType::Literal) => self.inp(addr),
                OpCode::INP(addr, AddressType::Pointer) => {
                    self.inp(Some(self.load(addr.unwrap())))
                }
                OpCode::HLT(_, _) => RuntimeState::Halted,
                OpCode::COB(_, _) => RuntimeState::Halted,
                OpCode::DAT(_, _) => RuntimeState::Halted, //should DAT be treated as the end of the program?
                OpCode::SOUT(addr, AddressType::Literal) => self.sout(addr),
                OpCode::SOUT(addr, AddressType::Pointer) => {
                    self.sout(Some(self.load(addr.unwrap())))
                }
                OpCode::POP(addr, AddressType::Literal) => self.pop(addr),
                OpCode::POP(addr, AddressType::Pointer) => {
                    self.pop(Some(self.load(addr.unwrap())))
                }
                OpCode::PUSH(addr, AddressType::Literal) => self.push(addr),
                OpCode::PUSH(addr, AddressType::Pointer) => {
                    self.push(Some(self.load(addr.unwrap())))
                }
            }
        } else {
            let common = self.get_common();
            RuntimeState::Error(RuntimeError::InvalidInstruction(
                common.program_counter,
                self.load(common.program_counter),
            ))
        }
    }
    fn get_current_instruction(&self) -> (Option<OpCode>, u16) {
        let literal = self.load(self.get_common().program_counter);
        let current_instruction = OpCode::try_from(literal);
        if let Ok(current_instruction) = current_instruction {
            (Some(current_instruction), literal)
        } else {
            (None, literal)
        }
    }
    fn start(&mut self) {
        while self.evaluate_current().is_running() {}
    }
}

pub trait RuntimeWithStack: Runtime {}
