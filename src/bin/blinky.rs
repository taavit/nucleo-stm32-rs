#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::AtomicBool;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed, Input, Pull, AnyPin};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::peripherals::PC13;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

static SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static SLOW_BLINK: Duration = Duration::from_secs(1);
static FAST_BLINK: Duration = Duration::from_millis(300);

static CURRENT: AtomicBool = AtomicBool::new(false);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(p.PA5, Level::High, Speed::Low);

    let led1 = Output::new(p.PC0, Level::High, Speed::Low);
    let led2 = Output::new(p.PC1, Level::High, Speed::Low);
    let led3 = Output::new(p.PC2, Level::High, Speed::Low);
    let led4 = Output::new(p.PC3, Level::High, Speed::Low);

    let button = Input::new(p.PC13, Pull::Down);
    let button = ExtiInput::new(button, p.EXTI13);

    spawner.spawn(button_task(button)).unwrap();
    spawner.spawn(blink_task(0, led.degrade())).unwrap();
    spawner.spawn(blink_task(1, led1.degrade())).unwrap();
    spawner.spawn(blink_task(2, led2.degrade())).unwrap();
    spawner.spawn(blink_task(3, led3.degrade())).unwrap();
    spawner.spawn(blink_task(4, led4.degrade())).unwrap();

    loop {
        let received_counter = SIGNAL.wait().await;
        CURRENT.swap(received_counter, core::sync::atomic::Ordering::Relaxed);

        info!("signalled, counter: {}", if received_counter { "PRESSED" } else {"DEPRESSO"});
    }
}

#[embassy_executor::task]
async fn button_task(mut pin: ExtiInput<'static, PC13>) {
    loop {
        pin.wait_for_low().await;
        info!("Button pressed!");
        SIGNAL.signal(true);
        pin.wait_for_high().await;
        info!("Button released!");
        SIGNAL.signal(false);
    }
}

#[embassy_executor::task(pool_size = 5)]
async fn blink_task(n: u8, mut led: Output<'static, AnyPin>) {
    loop {
        let fast_mode = CURRENT.load(core::sync::atomic::Ordering::Relaxed);
        if fast_mode {
            Timer::after(FAST_BLINK).await;
            led.set_high();
            Timer::after(FAST_BLINK).await;
            led.set_low();
            info!("Ping {}", n);
        } else {
            Timer::after(SLOW_BLINK).await;
            led.set_high();
            Timer::after(SLOW_BLINK).await;
            led.set_low();
            info!("Pong {}", n);
        }
    }
}
