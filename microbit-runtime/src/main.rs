#![no_std]
#![no_main]

mod letter;

extern crate alloc;

use crate::letter::number_to_image;
use alloc::string::ToString;
use core::panic::PanicInfo;
use cortex_m::asm::nop;
use cortex_m_rt::entry;
use microbit::display::blocking::Display;
use microbit::hal::timer::Instance;
use microbit::hal::Timer;
use microbit::Board;
use rtt_target::{rprintln, rtt_init_print};
use shared::runtime::{Runtime, RuntimeCommon, RuntimeState};
use shared::Mailbox;

pub struct MicrobitRuntime<T: Instance> {
    pub common: RuntimeCommon,
    pub display: Display,
    pub timer: Timer<T>,
}

impl<T: Instance> MicrobitRuntime<T> {
    pub fn new(p0: Mailbox, display: Display, timer: Timer<T>) -> Self {
        Self {
            common: RuntimeCommon {
                accumulator: 0,
                program_counter: 0,
                negative_flag: false,
                mailbox: p0,
            },
            timer,
            display,
        }
    }
}

impl<T: Instance> Runtime for MicrobitRuntime<T> {
    fn get_common(&self) -> &RuntimeCommon {
        &self.common
    }
    fn get_common_mut(&mut self) -> &mut RuntimeCommon {
        &mut self.common
    }
    fn sta(&mut self, addr: Option<u16>) -> RuntimeState {
        if let Some(addr) = addr {
            let v = self.common.accumulator;
            self.common.mailbox[addr as usize] = v;
            if addr == 99 {
                rprintln!("{:?}", v.to_string().chars());
                for i in v.to_string().chars() {
                    rprintln!("{}", i);
                    let num: u8 = i.to_digit(10).unwrap() as u8;
                    self.display
                        .show(&mut self.timer, number_to_image(num), 1000);
                }
            } else if addr == 98 && v == 1 {
                self.display
                    .show(&mut self.timer, number_to_image(10), 1000);
            }
            RuntimeState::Running
        } else {
            panic!("missing address")
        }
    }
    fn inp(&mut self, _: Option<u16>) -> RuntimeState {
        self.common.accumulator = 10;
        RuntimeState::Running
    }

    fn out(&mut self, _: Option<u16>) -> RuntimeState {
        rprintln!("{}", self.common.accumulator);
        RuntimeState::Running
    }
    fn sout(&mut self, _: Option<u16>) -> RuntimeState {
        let char = u8::try_from(self.common.accumulator)
            .expect("Cannot be converted to ascii character") as char;
        rprintln!("{}", char);
        RuntimeState::Running
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let timer = Timer::new(board.TIMER1);
    let display = Display::new(board.display_pins);
    let program = include_bytes!("../include/program.bin");
    rprintln!("{:?}", program);
    let mailbox = Mailbox::read_from_u8_slice(program);
    let mut runtime = MicrobitRuntime::new(mailbox.unwrap(), display, timer);
    runtime.start();
    loop {
        nop();
    }
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("{}", _info.to_string());
    loop {}
}
