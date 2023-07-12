use bitvec::{array::BitArray, order::Msb0};

use super::hal;

pub struct Radio<P, D> {
    pin: hal::port::Pin<hal::port::mode::Output, P>,
    delay: D,
}

impl<P, D> Radio<P, D>
where
    P: hal::port::PinOps,
    D: embedded_hal::blocking::delay::DelayUs<u16>,
{
    pub fn new(pin: hal::port::Pin<hal::port::mode::Output, P>, delay: D) -> Self {
        Self { pin, delay }
    }

    fn pulse(&mut self, on_us: u16, off_us: u16) {
        self.pin.set_high();
        self.delay.delay_us(on_us);
        self.pin.set_low();
        self.delay.delay_us(off_us);
    }

    fn start(&mut self) {
        self.pulse(500, 9000);
    }

    fn zero(&mut self) {
        self.pulse(500, 2000);
    }

    fn one(&mut self) {
        self.pulse(500, 4000);
    }

    fn stop(&mut self) {
        self.pulse(500, 500);
    }

    pub fn transmit<const N: usize>(&mut self, data: [u8; N]) {
        let data = BitArray::<_, Msb0>::new(data);
        self.start();
        for b in data {
            if b {
                self.one();
            } else {
                self.zero();
            }
        }
        self.stop();
    }
}
