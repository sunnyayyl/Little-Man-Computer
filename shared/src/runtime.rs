use crate::mailbox::Mailbox;
use crate::opcodes::OpCode;
#[cfg(not(feature = "std"))]
use core::option::{Option, Option::None, Option::Some};
#[cfg(not(feature = "std"))]
use core::result::{Result, Result::Err, Result::Ok};

use paste::paste;
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
macro_rules! object_getter {
    ( $( ($object:ident, $object_type:ty) ),* ) => {
        paste!{
            $(
                fn [<get_ $object>](&self) -> &$object_type;
                fn [<get_ $object _mut>](&mut self) -> &mut $object_type;
            )*
        }

    };
}
#[macro_export]
macro_rules! create_getter {
    ( $( ($object:ident, $object_type:ty) ),* ) => {
        paste!{
            $(
                 fn [<get_ $object>](&self) -> &$object_type{&self.$object}
                fn [<get_ $object _mut>](&mut self) -> &mut $object_type{&mut self.$object}
            )*
        }
    };
}
pub trait Runtime {
    object_getter!(
        (accumulator, u16),
        (program_counter, u16),
        (negative_flag, bool),
        (mailbox, Mailbox)
    );
    fn inp(&mut self, addr: Option<u16>) -> RuntimeState;
    fn out(&mut self, addr: Option<u16>) -> RuntimeState;
    fn sout(&mut self, addr: Option<u16>) -> RuntimeState;
    fn sta(&mut self, addr: Option<u16>) -> RuntimeState {
        (*self.get_mailbox_mut())[addr.expect("STA requires an address") as usize] =
            *self.get_accumulator();
        RuntimeState::Running
    }
    fn get_addresses(&self, addr: u16) -> u16 {
        self.get_mailbox()[addr as usize]
    }
    fn evaluate_current(&mut self) -> RuntimeState {
        let current_instruction = OpCode::try_from(self.get_addresses(*self.get_program_counter()));
        if let Ok(current_instruction) = current_instruction {
            *self.get_program_counter_mut() += 1;
            match current_instruction {
                OpCode::ADD(addr) => {
                    let new_value = self.get_accumulator()
                        + self.get_addresses(addr.expect("ADD requires an address"));
                    *self.get_accumulator_mut() = wrap_between_valid_values(new_value);
                    *self.get_negative_flag_mut() = false;
                } // Should overflow result in a negative flag?
                OpCode::SUB(addr) => {
                    let current_box = self.get_addresses(addr.expect("SUB requires an address"));
                    if self.get_accumulator() < &current_box {
                        *self.get_negative_flag_mut() = true;
                        *self.get_accumulator_mut() = current_box - self.get_accumulator();
                    } else {
                        *self.get_accumulator_mut() -= current_box;
                    }
                }
                OpCode::STA(addr) => return self.sta(addr),
                OpCode::LDA(addr) => {
                    *self.get_accumulator_mut() =
                        (*self.get_mailbox_mut())[addr.expect("LDA required an address") as usize]
                }
                OpCode::BRA(addr) => {
                    *self.get_program_counter_mut() = addr.expect("BRA require an addresses")
                }
                OpCode::BRZ(addr) => {
                    if self.get_accumulator() == &0 && !self.get_negative_flag() {
                        // Should the negative flag be taken into account?
                        *self.get_program_counter_mut() = addr.expect("BRZ require an addresses");
                    }
                }
                OpCode::BRP(addr) => {
                    if !self.get_negative_flag() {
                        *self.get_program_counter_mut() = addr.expect("BRP require an addresses");
                    }
                }
                OpCode::OUT(addr) => return self.out(addr),
                OpCode::INP(addr) => return self.inp(addr),
                OpCode::HLT(_) => return RuntimeState::Halted,
                OpCode::COB(_) => return RuntimeState::Halted,
                OpCode::DAT(_) => return RuntimeState::Halted, //should DAT be treated as the end of the program?
                OpCode::SOUT(addr) => return self.sout(addr),
            }
            RuntimeState::Running
        } else {
            RuntimeState::Error(RuntimeError::InvalidInstruction(
                *self.get_program_counter(),
                self.get_addresses(*self.get_program_counter()),
            ))
        }
    }
    fn get_current_instruction(&self) -> (Option<OpCode>, u16) {
        let literal = self.get_addresses(*self.get_program_counter());
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
