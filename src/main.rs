#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::{Duration, Timer};
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
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());
    let mut button = Input::new(p.PIN_13, Pull::Down);
    let mut led0 = Output::new(p.PIN_0, Level::Low);

    let mut state = State::First;
    loop {
        state = match state {
            State::First => State::Fast,
            State::Fast => fast_state(&mut button, &mut led0).await,
            State::Slow => slow_state(&mut button, &mut led0).await,
            State::AlwaysOn => always_on_state(&mut button, &mut led0).await,
            State::AlwaysOff => always_off_state(&mut button, &mut led0).await,
            State::Last => State::First,
        };
    }
}

async fn fast_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
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

async fn slow_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
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

async fn always_on_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
    led0.set_high();
    button.wait_for_falling_edge().await;
    State::AlwaysOff
}

async fn always_off_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
    led0.set_low();
    button.wait_for_falling_edge().await;
    State::Last
}
