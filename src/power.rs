use super::hal;

pub enum SleepMode {
    Idle = 0b000,
    AdcNoiseReduction = 0b001,
    PowerDown = 0b010,
}

#[cfg(feature = "atmega328p")]
fn sleep_reg(cpu: &hal::pac::CPU) -> &hal::pac::cpu::SMCR {
    &cpu.smcr
}

#[cfg(feature = "attiny85")]
fn sleep_reg(cpu: &hal::pac::CPU) -> &hal::pac::cpu::MCUCR {
    &cpu.mcucr
}

pub fn sleep_enable(cpu: &hal::pac::CPU, mode: SleepMode) {
    sleep_reg(cpu).modify(|_, w| w.sm().bits(mode as u8).se().set_bit());
}

pub fn sleep_disable(cpu: &hal::pac::CPU) {
    sleep_reg(cpu).modify(|_, w| w.se().clear_bit());
}
