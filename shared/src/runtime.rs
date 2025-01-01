use crate::mailbox::Mailbox;
use crate::opcodes::OpCode;
#[cfg(not(feature = "std"))]
use core::option::{Option, Option::None, Option::Some};
#[cfg(not(feature = "std"))]
use core::result::Result::Ok;

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

pub struct RuntimeCommon {
    pub accumulator: u16,
    pub program_counter: u16,
    pub negative_flag: bool,
    pub mailbox: Mailbox,
}
pub trait Runtime {
    fn get_common(&self) -> &RuntimeCommon;
    fn get_common_mut(&mut self) -> &mut RuntimeCommon;
    fn get_addresses(&self, addr: u16) -> u16 {
        self.get_common().mailbox[addr as usize]
    }
    fn add(&mut self, addr: Option<u16>) -> RuntimeState {
        let new_value = self.get_common().accumulator
            + self.get_addresses(addr.expect("ADD requires an address"));
        let common = self.get_common_mut();
        common.accumulator = wrap_between_valid_values(new_value);
        common.negative_flag = false;
        RuntimeState::Running
    }
    fn sub(&mut self, addr: Option<u16>) -> RuntimeState {
        let current_box = self.get_addresses(addr.expect("SUB requires an address"));
        let common = self.get_common_mut();
        if common.accumulator < current_box {
            common.negative_flag = true;
            common.accumulator = current_box - common.accumulator;
        } else {
            common.accumulator -= current_box;
        } // Should overflow result in a negative flag?
        RuntimeState::Running
    }
    fn sta(&mut self, addr: Option<u16>) -> RuntimeState {
        let common = self.get_common_mut();
        common.mailbox[addr.expect("STA requires an address") as usize] = common.accumulator;
        RuntimeState::Running
    }
    fn lda(&mut self, addr: Option<u16>) -> RuntimeState {
        let common = self.get_common_mut();
        common.accumulator = common.mailbox[addr.expect("LDA required an address") as usize];
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

    fn evaluate_current(&mut self) -> RuntimeState {
        let current_instruction =
            OpCode::try_from(self.get_addresses(self.get_common().program_counter));
        if let Ok(current_instruction) = current_instruction {
            self.get_common_mut().program_counter += 1;
            match current_instruction {
                OpCode::ADD(addr) => self.add(addr),
                OpCode::SUB(addr) => self.sub(addr),
                OpCode::STA(addr) => self.sta(addr),
                OpCode::LDA(addr) => self.lda(addr),
                OpCode::BRA(addr) => self.bra(addr),
                OpCode::BRZ(addr) => self.brz(addr),
                OpCode::BRP(addr) => self.brp(addr),
                OpCode::OUT(addr) => self.out(addr),
                OpCode::INP(addr) => self.inp(addr),
                OpCode::HLT(_) => return RuntimeState::Halted,
                OpCode::COB(_) => return RuntimeState::Halted,
                OpCode::DAT(_) => return RuntimeState::Halted, //should DAT be treated as the end of the program?
                OpCode::SOUT(addr) => self.sout(addr),
            }
        } else {
            let common = self.get_common();
            RuntimeState::Error(RuntimeError::InvalidInstruction(
                common.program_counter,
                self.get_addresses(common.program_counter),
            ))
        }
    }
    fn get_current_instruction(&self) -> (Option<OpCode>, u16) {
        let literal = self.get_addresses(self.get_common().program_counter);
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
