#![no_std]
#![no_main]

use core::array;

use defmt::unwrap;
use embassy_executor::{Executor, Spawner};
use embassy_futures::select::{select, Either};
use embassy_rp::{
    gpio,
    multicore::{spawn_core1, Stack},
    peripherals::CORE1,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;
use gpio::Level;
use static_cell::StaticCell;

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
use {defmt_rtt as _, panic_probe as _}; // Adjust the import path according to your setup

#[embassy_executor::main]
async fn main(_spawner0: Spawner) {
    let pins = Pins::new();

    let button = pins.button;
    let led0 = pins.led0;

    loop {
        led0.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led0.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}

struct Pins {
    button: &'static mut gpio::Input<'static>,
    led0: &'static mut gpio::Output<'static>,
}

impl Pins {
    fn new() -> Self {
        let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

        static BUTTON_PIN: StaticCell<gpio::Input> = StaticCell::new();
        let button = BUTTON_PIN.init(gpio::Input::new(p.PIN_13, gpio::Pull::Down));

        static LED0_PIN: StaticCell<gpio::Output> = StaticCell::new();
        let led0 = LED0_PIN.init(gpio::Output::new(p.PIN_0, Level::Low));

        Self { button, led0 }
    }
}
