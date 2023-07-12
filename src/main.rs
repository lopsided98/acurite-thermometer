#![no_std]
#![no_main]

use atmega_hal as hal;
use embedded_hal::blocking::delay::DelayMs;
use panic_halt as _;

type Delay = hal::delay::Delay<hal::clock::MHz16>;

#[avr_device::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let mut led = pins.pb5.into_output();

    loop {
        led.toggle();
        Delay::new().delay_ms(1000u16);
    }
}
