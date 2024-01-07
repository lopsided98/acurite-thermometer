use hal::wdt::WdtOps;

use super::hal;

#[derive(Clone, Copy)]
pub struct Config(u8);

impl Config {
    pub const fn new() -> Self {
        Self(1 << 4 /* WDCE */)
    }

    pub const fn enable(mut self) -> Self {
        self.0 |= 1 << 3;
        self
    }

    pub const fn timeout(mut self, timeout: hal::wdt::Timeout) -> Self {
        self.0 &= !(0b00010111);
        use hal::wdt::Timeout::*;
        let wdp = match timeout {
            Ms16 => 0,
            Ms32 => 1,
            Ms64 => 2,
            Ms125 => 3,
            Ms250 => 4,
            Ms500 => 5,
            Ms1000 => 6,
            Ms2000 => 7,
            Ms4000 => 8,
            Ms8000 => 9,
        };
        self.0 |= (wdp & 0b1000) << 2;
        self.0 |= wdp & 0b0111;
        self
    }

    pub const fn interrupt(mut self) -> Self {
        self.0 |= 1 << 6;
        self
    }
}

pub struct Watchdog {
    wdt: hal::pac::WDT,
}

impl Watchdog {
    pub fn new(mut wdt: hal::pac::WDT, mcusr: &hal::pac::cpu::MCUSR) -> Self {
        wdt.raw_init(mcusr);
        Self { wdt }
    }

    pub fn configure(&mut self, config: Config) {
        // Enable watchdog configuration mode
        self.wdt.wdtcr.write(|w| w.wdce().set_bit().wde().set_bit());
        // Apply config
        self.wdt.wdtcr.write(|w| unsafe { w.bits(config.0) });
    }
}
