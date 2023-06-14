#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{adc::Adc, gpio::{Level, Output, Speed}};
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PA0;

    let mut vrefint = adc.enable_vref(&mut Delay);
    let vrefint_sample = adc.read(&mut vrefint);
    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/CD00161566.pdf
        // 5.3.4 Embedded reference voltage
        const VREFINT_MV: u32 = 1200; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    let mut led1 = Output::new(p.PC0, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PC1, Level::High, Speed::Low);
    let mut led3 = Output::new(p.PC2, Level::High, Speed::Low);
    let mut led4 = Output::new(p.PC3, Level::High, Speed::Low);

    loop {
        let v = adc.read(&mut pin);
        led1.set_level(if v > 3072 { Level::High } else { Level::Low });
        led2.set_level(if v > 2048 { Level::High } else { Level::Low });
        led3.set_level(if v > 1024 { Level::High } else { Level::Low });
        led4.set_level(if v > 512 { Level::High } else { Level::Low });
        info!("--> {} - {} mV", v, convert_to_millivolts(v));
        Timer::after(Duration::from_millis(100)).await;
    }
}
