#![no_std]
#![no_main]

use atmega_hal as hal;
use bitflags::bitflags;
use embedded_hal::{
    blocking::delay::DelayMs,
    blocking::i2c::{Write, WriteRead},
};
use panic_halt as _;

type Speed = hal::clock::MHz16;
type Delay = hal::delay::Delay<Speed>;

const TMP102_ADDR: u8 = 0x48;

const TMP102_TEMPERATURE_REG: u8 = 0x0;
const TMP102_CONFIG_REG: u8 = 0x1;

bitflags! {
    #[repr(transparent)]
    struct Tmp102Config: u16 {
        const OS = 1 << 15;
        const R1 = 1 << 14;
        const R2 = 1 << 13;
        const F1 = 1 << 12;
        const F2 = 1 << 11;
        const POL = 1 << 10;
        const TM = 1 << 9;
        const SD = 1 << 8;
        const CR1 = 1 << 7;
        const CR0 = 1 << 6;
        const AL = 1 << 5;
        const EM = 1 << 4;
    }
}

impl Tmp102Config {
    const fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bits_retain(u16::from_be_bytes(bytes))
    }
}

/// TMP102 config
/// - One-shot
/// - Shutdown
/// - Extended mode
const TMP102_CONFIG: Tmp102Config = Tmp102Config::OS
    .intersection(Tmp102Config::SD)
    .intersection(Tmp102Config::EM);

fn tmp102_reg_write<W: Write>(i2c: &mut W, reg: u8, value: u16) -> Result<(), W::Error> {
    let value = value.to_be_bytes();
    i2c.write(TMP102_ADDR, &[reg, value[0], value[1]])
}

#[inline]
fn i2c_reg_read<W: WriteRead, const N: usize>(
    i2c: &mut W,
    address: u8,
    reg: u8,
) -> Result<[u8; N], W::Error> {
    let mut value = [0u8; N];
    i2c.write_read(address, &[reg], &mut value)?;
    Ok(value)
}

const fn convert_temperature(temp_reg: i16) -> i16 {
    let temp_whole = temp_reg / 256;
    // Binary fractional part, out of 16
    let temp_frac_bin = ((temp_reg & 0xff) >> 4) as u8;
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
    let pins = hal::pins!(dp);

    let mut led = pins.pb5.into_output();

    let i2c_sda = pins.pc4;
    let i2c_scl = pins.pc5;

    let mut i2c = hal::I2c::<Speed>::new(
        dp.TWI,
        i2c_sda.into_pull_up_input(),
        i2c_scl.into_pull_up_input(),
        50000,
    );

    loop {
        tmp102_reg_write(&mut i2c, TMP102_CONFIG_REG, TMP102_CONFIG.bits()).unwrap();
        // A single conversion typically takes 26 ms
        Delay::new().delay_ms(30u16);
        loop {
            let config = Tmp102Config::from_bytes(i2c_reg_read(&mut i2c, TMP102_ADDR, TMP102_CONFIG_REG).unwrap());
            if config.contains(Tmp102Config::OS) {
                break;
            }
            Delay::new().delay_ms(2u16);
        }

        let temp_reg = i16::from_be_bytes(
            i2c_reg_read(&mut i2c, TMP102_ADDR, TMP102_TEMPERATURE_REG).unwrap(),
        );
        let temp = convert_temperature(temp_reg);

        led.toggle();
        Delay::new().delay_ms(temp as u16);
    }
}
