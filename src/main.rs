#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio;
use embassy_time::{Duration, Timer};
use gpio::Level;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

enum State {
    First,
    Fast,
    Slow,
    AlwaysOn,
    AlwaysOff,
    Last,
}

#[embassy_executor::main]
async fn main(_spawner0: Spawner) {
    let pins = Pins::new();
    let button = pins.button;
    let led0 = pins.led0;

    let mut state = State::First;
    loop {
        state = match state {
            State::First => State::Fast,
            State::Fast => fast_state(button, led0).await,
            State::Slow => slow_state(button, led0).await,
            State::AlwaysOn => always_on_state(button, led0).await,
            State::AlwaysOff => always_off_state(button, led0).await,
            State::Last => State::First,
        };
    }
}

type Button = gpio::Input<'static>;
type Led = gpio::Output<'static>;

async fn fast_state(button: &mut Button, led0: &mut Led) -> State {
    loop {
        led0.toggle();
        if let Either::Second(()) = select(
            Timer::after(Duration::from_millis(100)),
            button.wait_for_falling_edge(),
        )
        .await
        {
            return State::Slow;
        }
    }
}

async fn slow_state(button: &mut Button, led0: &mut Led) -> State {
    loop {
        led0.toggle();
        if let Either::Second(()) = select(
            Timer::after(Duration::from_millis(500)),
            button.wait_for_falling_edge(),
        )
        .await
        {
            return State::AlwaysOn;
        }
    }
}

async fn always_on_state(button: &mut Button, led0: &mut Led) -> State {
    led0.set_high();
    button.wait_for_falling_edge().await;
    State::AlwaysOff
}

async fn always_off_state(button: &mut Button, led0: &mut Led) -> State {
    led0.set_low();
    button.wait_for_falling_edge().await;
    State::Last
}

struct Pins {
    button: &'static mut Button,
    led0: &'static mut Led,
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
