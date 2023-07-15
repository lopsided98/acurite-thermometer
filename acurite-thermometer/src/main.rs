#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

#[cfg(feature = "atmega328p")]
pub use atmega_hal as hal;
#[cfg(feature = "attiny85")]
pub use attiny_hal as hal;
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
        pub random: atmega_hal::port::PC3 = pc3,
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

fn random_u8(adc: &mut adc::Adc, channel: hal::pac::adc::admux::MUX_A) -> u8 {
    adc.enable_pin(channel);
    let mut value = 0;
    for _ in 0..8 {
        value <<= 1;
        value |= (adc.read_blocking(channel) as u8) & 0x1;
    }
    value
}

fn read_battery_mv(adc: &mut adc::Adc, cpu: &hal::pac::CPU) -> u16 {
    let value = adc.read_blocking_noise_reduction(hal::pac::adc::admux::MUX_A::ADC_VBG, cpu);
    ((1.1 * 1023.0 * 1000.0) as u32 / value as u32) as u16
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
    watchdog.start(hal::wdt::Timeout::Ms2000).unwrap();
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
    let id = random_u8(&mut adc, hal::pac::adc::admux::MUX_A::ADC3);
    pins.random.into_pull_up_input();

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
        led.set_high();
        let Ok(temp_reg) = sensor.oneshot(TMP102_CONFIG) else {
            #[cfg(feature = "atmega328p")]
            ufmt::uwriteln!(&mut uart, "Failed to read temperature").void_unwrap();
            continue;
        };
        led.set_low();
        let temp = acurite_protocol::tx0606::convert_temperature(temp_reg);

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

        let message = acurite_protocol::tx0606::generate(id, battery_mv > BATTERY_LOW_MV, temp);

        for _ in 0..7 {
            radio.transmit(message);
        }

        adc.enable(false);
        power::sleep_enable(&dp.CPU, power::SleepMode::PowerDown);
        // 15x2=30 sec wakeups
        for _ in 0..15 {
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
