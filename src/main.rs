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
    // loop until button is pressed down
    loop {
        led0.toggle();
        if let Either::First(()) = select(
            button.wait_for_rising_edge(), // wait for button press down
            Timer::after(Duration::from_millis(100)),
        )
        .await
        {
            break;
        }
    }

    // if button is released within 1 second, go to slow state else go to start state
    if_fast_release_else(button, State::Slow, State::First).await
}

async fn if_fast_release_else(
    button: &mut Input<'_>,
    tap_state: State,
    hold_state: State,
) -> State {
    if let Either::First(()) = select(
        button.wait_for_falling_edge(), // wait for button release
        Timer::after(Duration::from_secs(1)),
    )
    .await
    {
        tap_state
    } else {
        button.wait_for_falling_edge().await; // wait for button release
        hold_state
    }
}

async fn slow_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
    loop {
        led0.toggle();
        if let Either::First(()) = select(
            button.wait_for_rising_edge(),
            Timer::after(Duration::from_millis(500)),
        )
        .await
        {
            break;
        }
    }
    if_fast_release_else(button, State::AlwaysOn, State::First).await
}

async fn always_on_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
    led0.set_high();
    button.wait_for_rising_edge().await;
    if_fast_release_else(button, State::AlwaysOff, State::First).await
}

async fn always_off_state(button: &mut Input<'_>, led0: &mut Output<'_>) -> State {
    led0.set_low();
    button.wait_for_rising_edge().await;
    if_fast_release_else(button, State::Last, State::First).await
}
