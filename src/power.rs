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

#[inline]
pub fn disable_bod_in_sleep(cpu: &hal::pac::CPU) {
    let mcucr = cpu.mcucr.read().bits();
    cpu.mcucr
        .write(|w| unsafe { w.bits(mcucr) }.bods().set_bit().bodse().set_bit());
    cpu.mcucr.write(|w| {
        unsafe { w.bits(mcucr) }
            .bods()
            .set_bit()
            .bodse()
            .clear_bit()
    });
}

pub fn disable_unused_hardware(cpu: &hal::pac::CPU, ac: &hal::pac::AC) {
    // Disable timers and SPI
    cpu.prr.write(|w| {
        w.prtim0().set_bit().prtim1().set_bit();
        #[cfg(feature = "atmega328p")]
        w.prspi().set_bit().prtim2().set_bit();
        w
    });

    // Disable analog comparator
    ac.acsr.modify(|_, w| w.acd().set_bit());
}

/// Set the CPU clock divider to obtain a desired frequency, assuming the clock
/// is supplied by an external 16 MHz crystal. If the clock frequency is not ach
#[cfg(feature = "atmega328p")]
pub fn cpu_clock_divider<InputClock, CpuClock>(cpu: &hal::pac::CPU) -> Result<(), ()>
where
    InputClock: hal::clock::Clock,
    CpuClock: hal::clock::Clock,
{
    if CpuClock::FREQ > InputClock::FREQ {
        return Err(());
    }
    let divider = InputClock::FREQ / CpuClock::FREQ;
    let remainder = InputClock::FREQ % CpuClock::FREQ;
    if remainder != 0 || divider > 256 {
        return Err(());
    }
    let clkps = divider.trailing_zeros() as u8;

    avr_device::interrupt::free(|_| {
        cpu.clkpr.write(|w| w.clkpce().set_bit());
        cpu.clkpr.write(|w| unsafe { w.clkps().bits(clkps) });
    });
    Ok(())
}
