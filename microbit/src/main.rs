#![no_std]
#![no_main]
extern crate alloc;
use cortex_m::asm::nop;
use cortex_m_rt::entry;
use nrf51_hal as hal;
#[allow(unused_imports)]
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use embedded_hal::digital::OutputPin;
use shared::create_getter;
use shared::Mailbox;
use shared::runtime::{Runtime, RuntimeState};
use paste::paste;
use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();
pub struct MicrobitRuntime {
    pub accumulator: u16,
    pub program_counter: u16,
    pub negative_flag: bool,
    pub mailbox: Mailbox,
    //pub peripherals: Peripherals,
}

impl MicrobitRuntime {
    pub fn new(p0: Mailbox) -> Self {
        Self {
            accumulator: 0,
            program_counter: 0,
            negative_flag: false,
            mailbox: p0,
        }
    }
}

impl Runtime for MicrobitRuntime {
    create_getter!(
        (accumulator, u16),
        (program_counter, u16),
        (negative_flag, bool),
        (mailbox, Mailbox)
    );
    fn inp(&mut self, _: Option<u16>) -> RuntimeState {
        self.accumulator=10;
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
        if let Some(addr)=addr{
            (*self.get_mailbox_mut())[addr as usize] =
                *self.get_accumulator();
            if addr==99{
                //self.peripherals.
            }
            RuntimeState::Running
        }else{
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
    let p=hal::pac::Peripherals::take().unwrap();
    let port0=hal::gpio::p0::Parts::new(p.GPIO);
    let mut row1=port0.p0_13.into_push_pull_output(hal::gpio::Level::Low);
    let mut col1=port0.p0_04.into_push_pull_output(hal::gpio::Level::Low);
    //  col1.set_high().unwrap();
    row1.set_high().unwrap();
    let program=include_bytes!("../include/program.bin");
    let mailbox = Mailbox::read_from_u8_slice(program);
    let mut runtime = MicrobitRuntime::new(mailbox.unwrap());
    runtime.start();
    loop {
        nop();
    }
}
