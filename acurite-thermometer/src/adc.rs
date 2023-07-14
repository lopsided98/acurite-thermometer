use hal::adc::AdcOps;

use crate::power;

use super::hal;

pub struct Adc {
    adc: hal::pac::ADC,
}

impl Adc {
    pub fn new(adc: hal::pac::ADC, settings: hal::adc::AdcSettings) -> Self {
        let mut s = Self { adc };
        s.initialize(settings);
        s
    }

    pub fn initialize(&mut self, settings: hal::adc::AdcSettings) {
        self.adc.raw_init(settings);
    }

    pub fn enable_pin(
        &mut self,
        channel: <hal::pac::ADC as hal::adc::AdcOps<super::Hal>>::Channel,
    ) {
        self.adc.raw_enable_channel(channel);
    }

    pub fn read_blocking(
        &mut self,
        channel: <hal::pac::ADC as hal::adc::AdcOps<super::Hal>>::Channel,
    ) -> u16 {
        self.adc.raw_set_channel(channel);
        self.adc.raw_start_conversion();
        while self.adc.raw_is_converting() {}
        self.adc.raw_read_adc()
    }

    pub fn read_blocking_noise_reduction(
        &mut self,
        channel: <hal::pac::ADC as hal::adc::AdcOps<super::Hal>>::Channel,
        cpu: &hal::pac::CPU,
    ) -> u16 {
        self.adc.raw_set_channel(channel);
        power::sleep_enable(cpu, power::SleepMode::AdcNoiseReduction);
        loop {
            avr_device::asm::sleep();
            if !self.adc.raw_is_converting() {
                break;
            }
        }
        power::sleep_disable(cpu);
        self.adc.raw_read_adc()
    }

    pub fn enable(&mut self, enable: bool) {
        self.adc.adcsra.modify(|_, w| w.aden().bit(enable));
    }

    pub fn interrupt(&mut self, enable: bool) {
        self.adc.adcsra.modify(|_, w| w.adie().bit(enable));
    }
}
