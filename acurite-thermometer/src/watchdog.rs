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
        self.wdt.wdtcsr.modify(|_, w| w.wdie().bit(enable));
    }
}
