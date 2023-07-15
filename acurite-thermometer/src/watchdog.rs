use hal::wdt::WdtOps;

use super::hal;

pub struct Watchdog {
    wdt: hal::pac::WDT,
}

impl Watchdog {
    pub fn new(mut wdt: hal::pac::WDT, mcusr: &hal::pac::cpu::MCUSR) -> Self {
        wdt.raw_init(mcusr);
        Self { wdt }
    }

    pub fn start(&mut self, timeout: hal::wdt::Timeout) -> Result<(), ()> {
        self.wdt.raw_start(timeout)
    }

    pub fn interrupt(&mut self, enable: bool) {
        #[cfg(feature = "atmega328p")]
        let reg = &self.wdt.wdtcsr;
        #[cfg(feature = "attiny85")]
        let reg = &self.wdt.wdtcr;
        reg.modify(|_, w| w.wdie().bit(enable));
        reg.modify(|_, w| w.wdie().bit(enable));
    }
}
