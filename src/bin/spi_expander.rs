#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{spi::{Config, Spi}, dma::NoDma, time::Hertz, gpio::{Output, Level, Speed, AnyPin}, peripherals::SPI1};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

/// Tested on MCP23S08
enum ExpanderAddress {
    McpIodir =	    0x00,
    // MCP_IPOL =	    0x01,
    // MCP_GPINTEN =	0x02,
    // MCP_DEFVAL =	0x03,
    // MCP_INTCON =	0x04,
    // MCP_IOCON =	    0x05,
    // MCP_GPPU =	    0x06,
    // MCP_INTF =	    0x07,
    // MCP_INTCAP =	0x08,
    // McpGpio =	    0x09,
    McpOlat =	    0x0a,
}

struct ExpanderSpi {
    spi: Spi<'static, SPI1, NoDma, NoDma>,
    cs: Output<'static, AnyPin>,
}

impl ExpanderSpi {
    fn write(&mut self, address: ExpanderAddress, value: u8) {
        self.cs.set_low();
        self.spi.blocking_transfer_in_place(&mut [0x40u8, address as u8, value]).unwrap();
        self.cs.set_high();
    }
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let spi = Spi::new(
        p.SPI1,
        p.PA5,
        p.PA7,
        p.PA6,
        NoDma,
        NoDma,
        Hertz(1_000_000),
        Config::default(),
    );

    let cs = Output::new(p.PC0, Level::High, Speed::VeryHigh);

    let cs_deg = cs.degrade();
    let mut exp = ExpanderSpi { spi, cs: cs_deg };

    exp.write(ExpanderAddress::McpIodir, !0x01);

    loop {
        Timer::after(Duration::from_millis(250)).await;
        exp.write(ExpanderAddress::McpOlat, 0x01);
        Timer::after(Duration::from_millis(250)).await;
        exp.write(ExpanderAddress::McpOlat, 0x00);
        info!("Migam!");
    }
}
