#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

#[cfg(feature = "atmega328p")]
pub use atmega_hal as hal;
#[cfg(feature = "attiny85")]
pub use attiny_hal as hal;
use bitvec::view::BitView;
#[cfg(feature = "atmega328p")]
use hal::usart::BaudrateArduinoExt;
use hal::{port::Pin, prelude::*};
use panic_halt as _;

mod adc;
mod power;
mod radio;
mod tmp102;
mod watchdog;

#[cfg(feature = "atmega328p")]
type Hal = hal::Atmega;

#[cfg(feature = "attiny85")]
type Hal = hal::Attiny;

type Speed = hal::clock::MHz1;
type Delay = hal::delay::Delay<Speed>;

/// TMP102 config
/// - One-shot
/// - Shutdown
/// - Extended mode
const TMP102_CONFIG: tmp102::Config = tmp102::Config::OS
    .union(tmp102::Config::SD)
    .union(tmp102::Config::EM);

const BATTERY_LOW_MV: u16 = 2000;

#[cfg(feature = "atmega328p")]
avr_hal_generic::renamed_pins! {
    type Pin = Pin;

    pub struct Pins from atmega_hal::Pins {
        pub led: atmega_hal::port::PB5 = pb5,
        pub random: atmega_hal::port::PC0 = pc0,
        pub uart_rx: atmega_hal::port::PD0 = pd0,
        pub uart_tx: atmega_hal::port::PD1 = pd1,
        pub i2c_sda: atmega_hal::port::PC4 = pc4,
        pub i2c_scl: atmega_hal::port::PC5 = pc5,
        pub radio: atmega_hal::port::PB1 = pb1,
    }
}

#[cfg(feature = "attiny85")]
avr_hal_generic::renamed_pins! {
    type Pin = Pin;

    pub struct Pins from attiny_hal::Pins {
        pub led: attiny_hal::port::PB5 = pb5,
        pub random: attiny_hal::port::PB3 = pb3,
        pub i2c_sda: attiny_hal::port::PB0 = pb0,
        pub i2c_scl: attiny_hal::port::PB2 = pb2,
        pub radio: attiny_hal::port::PB4 = pb4,
    }
}

fn read_battery_mv(adc: &mut adc::Adc, cpu: &hal::pac::CPU) -> u16 {
    let value = adc.read_blocking_noise_reduction(hal::pac::adc::admux::MUX_A::ADC_VBG, cpu);
    ((1.1 * 1023.0 * 1000.0) as u32 / value as u32) as u16
}

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

const fn lfsr_sequence<const N: usize>() -> [u8; N] {
    let mut reg: u8 = 0x7C;
    let mut temp_reg: u8;
    let mut sequence = [0u8; N];

    let mut i: usize = 0;
    loop {
        if i >= N {
            break;
        }
        temp_reg = reg & 0x01;
        reg >>= 1;
        reg |= temp_reg << 7;

        if temp_reg != 0 {
            reg ^= 0x18
        }
        sequence[i] = reg;
        i += 1;
    }
    sequence
}

fn lfsr_hash<const N: usize, const M: usize, const OFFSET: usize>(data: &[u8; N]) -> u8 {
    let sequence = lfsr_sequence::<M>();

    let mut hash_reg: u8 = 0;
    for (byte_idx, byte) in data.iter().enumerate() {
        for (bit_idx, bit) in byte
            .view_bits::<bitvec::order::Msb0>()
            .into_iter()
            .enumerate()
        {
            if *bit {
                hash_reg ^= sequence[byte_idx * 8 + bit_idx + OFFSET]
            }
        }
    }

    hash_reg
}

fn message(id: u8, battery_ok: bool, temperature: i16) -> [u8; 4] {
    let status = if battery_ok { 0b1000 } else { 0b0000 };
    let body: [u8; 3] = [
        id,
        status << 4 | (temperature & 0xf00 >> 8) as u8,
        (temperature & 0xff) as u8,
    ];
    let hash = lfsr_hash::<3, 32, 4>(&body);
    [body[0], body[1], body[2], hash]
}

#[avr_device::interrupt(atmega328p)]
fn ADC() {}

#[avr_device::interrupt(atmega328p)]
fn WDT() {}

#[avr_device::entry]
fn main() -> ! {
    unsafe { avr_device::interrupt::enable() };

    let dp = hal::Peripherals::take().unwrap();

    // Set the CPU clock divider to match the configured speed
    #[cfg(feature = "atmega328p")]
    power::cpu_clock_divider::<hal::clock::MHz16, Speed>(&dp.CPU).unwrap();
    power::disable_unused_hardware(&dp.CPU, &dp.AC);

    let mut watchdog = watchdog::Watchdog::new(dp.WDT, &dp.CPU.mcusr);
    watchdog.start(hal::wdt::Timeout::Ms8000).unwrap();
    watchdog.interrupt(true);

    let pins = Pins::with_mcu_pins(hal::pins!(dp));

    // Custom ADC driver that allows the use of noise reduction mode
    let mut adc = adc::Adc::new(
        dp.ADC,
        hal::adc::AdcSettings {
            clock_divider: hal::adc::ClockDivider::Factor16,
            ref_voltage: hal::adc::ReferenceVoltage::AVcc,
        },
    );
    // Enable ADC interrupt for power-reduction mode
    adc.interrupt(true);

    // Random transmitter ID included in each message
    let id = {
        #[cfg(feature = "atmega328p")]
        let random_channel = hal::pac::adc::admux::MUX_A::ADC0;
        adc.enable_pin(random_channel);
        let id = adc.read_blocking(random_channel) as u8;
        pins.random.into_pull_up_input();
        id
    };

    let mut led = pins.led.into_output();

    #[cfg(feature = "atmega328p")]
    let mut uart = hal::usart::Usart0::<Speed>::new(
        dp.USART0,
        pins.uart_rx,
        pins.uart_tx.into_output(),
        9600.into_baudrate(),
    );

    let i2c = hal::I2c::<Speed>::with_external_pullup(dp.TWI, pins.i2c_sda, pins.i2c_scl, 20000);

    let mut sensor = tmp102::Tmp102::new(i2c, Delay::new());
    let mut radio = radio::Radio::new(pins.radio.into_output(), Delay::new());

    // The first ADC read seems to be bad, so discard it. Its not the bandgap,
    // since it still happens if you wait a long time.
    read_battery_mv(&mut adc, &dp.CPU);

    #[cfg(feature = "atmega328p")]
    ufmt::uwriteln!(&mut uart, "Booted").void_unwrap();

    loop {
        let Ok(temp_reg) = sensor.oneshot(TMP102_CONFIG) else {
            #[cfg(feature = "atmega328p")]
            ufmt::uwriteln!(&mut uart, "Failed to read temperature").void_unwrap();
            continue;
        };
        let temp = convert_temperature(temp_reg);

        let battery_mv = read_battery_mv(&mut adc, &dp.CPU);

        #[cfg(feature = "atmega328p")]
        ufmt::uwriteln!(
            &mut uart,
            "id: {}, temp: {}, reg: {}, batt: {}",
            id,
            temp,
            temp_reg,
            battery_mv
        )
        .void_unwrap();

        let message = message(id, battery_mv > BATTERY_LOW_MV, temp);

        led.set_high();
        for _ in 0..7 {
            radio.transmit(message);
        }
        led.set_low();

        adc.enable(false);
        power::sleep_enable(&dp.CPU, power::SleepMode::PowerDown);
        // Watchdog wakes up after 8s, so sleep 4 times to get 32 second
        // intervals.
        for _ in 0..4 {
            power::disable_bod_in_sleep(&dp.CPU);
            avr_device::asm::sleep();
            // If WDE is set, WDIE is automatically cleared by hardware when a
            // time-out occurs. This is useful for keeping the Watchdog Reset
            // security while using the interrupt. After the WDIE bit is cleared,
            // the next time-out will generate a reset. To avoid the Watchdog Reset,
            // WDIE must be set after each interrupt.
            watchdog.interrupt(true);
        }
        power::sleep_disable(&dp.CPU);
        adc.enable(true);
    }
}
