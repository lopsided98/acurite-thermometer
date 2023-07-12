#![no_std]
#![no_main]

pub use atmega_hal as hal;
use embedded_hal::blocking::delay::DelayMs;
use hal::{prelude::*, usart::BaudrateArduinoExt};
use panic_halt as _;

mod radio;
mod tmp102;

type Speed = hal::clock::MHz1;
type Delay = hal::delay::Delay<Speed>;

/// TMP102 config
/// - One-shot
/// - Shutdown
/// - Extended mode
const TMP102_CONFIG: tmp102::Config = tmp102::Config::OS
    .union(tmp102::Config::SD)
    .union(tmp102::Config::EM);

const fn convert_temperature(temp_reg: i16) -> i16 {
    let temp_whole = temp_reg >> 7;
    // Binary fractional part, out of 16
    let temp_frac_bin = ((temp_reg & 0x7f) >> 3) as u8;
    // Convert to decimal fraction, with proper rounding
    let temp_frac = match temp_frac_bin {
        0 => 0,  // 0
        1 => 1,  // 0.0625
        2 => 1,  // 0.125
        3 => 2,  // 0.1875
        4 => 3,  // 0.25
        5 => 3,  // 0.3125
        6 => 4,  // 0.375
        7 => 4,  // 0.4375
        8 => 5,  // 0.5
        9 => 6,  // 0.5625
        10 => 6, // 0.625
        11 => 7, // 0.6875
        12 => 8, // 0.75
        13 => 8, // 0.8125
        14 => 9, // 0.875
        15 => 9, // 0.9375
        _ => unreachable!(),
    };

    temp_whole * 10 + temp_frac as i16
}

#[avr_device::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();

    // Set CPU clock to 1 MHz
    let clkpr = &dp.CPU.clkpr;
    clkpr.write(|w| w.clkpce().set_bit());
    clkpr.write(|w| w.clkps().val_0x04());

    let pins = hal::pins!(dp);

    let mut led = pins.pb5.into_output();

    let mut uart = hal::usart::Usart0::<Speed>::new(
        dp.USART0,
        pins.pd0,
        pins.pd1.into_output(),
        9600.into_baudrate(),
    );

    let i2c_sda = pins.pc4;
    let i2c_scl = pins.pc5;

    let i2c = hal::I2c::<Speed>::with_external_pullup(dp.TWI, i2c_sda, i2c_scl, 20000);

    let mut sensor = tmp102::Tmp102::new(i2c, Delay::new());
    let mut radio = radio::Radio::new(pins.pb1.into_output(), Delay::new());

    loop {
        led.toggle();
        let Ok(temp_reg) = sensor.oneshot(TMP102_CONFIG) else {
            ufmt::uwriteln!(&mut uart, "Failed to read temperature").void_unwrap();
            continue;
        };
        let temp = convert_temperature(temp_reg);

        ufmt::uwriteln!(&mut uart, "Temp: {}, reg: {}", temp, temp_reg).void_unwrap();

        radio.transmit(temp.to_be_bytes());

        Delay::new().delay_ms(1000u16);
    }
}
