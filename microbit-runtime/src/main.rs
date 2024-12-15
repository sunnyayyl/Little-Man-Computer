#![no_std]
#![no_main]

mod letter;

extern crate alloc;

use crate::letter::number_to_image;
use alloc::string::ToString;
use core::panic::PanicInfo;
use cortex_m::asm::nop;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;
use microbit::display::blocking::Display;
use microbit::hal::timer::Instance;
use microbit::hal::Timer;
use microbit::Board;
use paste::paste;
use rtt_target::{rprintln, rtt_init_print};
use shared::runtime::{Runtime, RuntimeState};
use shared::{create_getter, Mailbox};

#[global_allocator]
static HEAP: Heap = Heap::empty();
pub struct MicrobitRuntime<T: Instance> {
    pub accumulator: u16,
    pub program_counter: u16,
    pub negative_flag: bool,
    pub mailbox: Mailbox,
    pub display: Display,
    pub timer: Timer<T>,
}

impl<T: Instance> MicrobitRuntime<T> {
    pub fn new(p0: Mailbox, display: Display, timer: Timer<T>) -> Self {
        Self {
            accumulator: 0,
            program_counter: 0,
            negative_flag: false,
            mailbox: p0,
            timer,
            display,
        }
    }
}

impl<T: Instance> Runtime for MicrobitRuntime<T> {
    create_getter!(
        (accumulator, u16),
        (program_counter, u16),
        (negative_flag, bool),
        (mailbox, Mailbox)
    );
    fn inp(&mut self, _: Option<u16>) -> RuntimeState {
        self.accumulator = 10;
        RuntimeState::Running
    }
    fn out(&mut self, _: Option<u16>) -> RuntimeState {
        rprintln!("{}", self.accumulator);
        RuntimeState::Running
    }

    fn sout(&mut self, _: Option<u16>) -> RuntimeState {
        let char =
            u8::try_from(self.accumulator).expect("Cannot be converted to ascii character") as char;
        rprintln!("{}", char);
        RuntimeState::Running
    }
    fn sta(&mut self, addr: Option<u16>) -> RuntimeState {
        if let Some(addr) = addr {
            let v = *self.get_accumulator();
            (*self.get_mailbox_mut())[addr as usize] = v;
            if addr == 99 {
                rprintln!("{:?}", v.to_string().chars());
                for i in v.to_string().chars() {
                    rprintln!("{}", i);
                    let num: u8 = i.to_digit(10).unwrap() as u8;
                    self.display
                        .show(&mut self.timer, number_to_image(num), 1000);
                }
            }else if addr == 98 && v == 1 {
                self.display
                    .show(&mut self.timer, number_to_image(10), 1000);
            }
            RuntimeState::Running
        } else {
            panic!("missing address")
        }
    }
}

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
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
