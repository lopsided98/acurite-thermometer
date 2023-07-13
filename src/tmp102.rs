use bitflags::bitflags;
use embedded_hal::blocking::{
    delay::DelayMs,
    i2c::{Write, WriteRead},
};

const ADDR: u8 = 0x48;

bitflags! {
    #[repr(transparent)]
    pub struct Config: u16 {
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

impl Config {
    const fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bits_retain(u16::from_be_bytes(bytes))
    }
}

#[repr(u8)]
enum Register {
    Temperature = 0x00,
    Config = 0x01,
}

pub struct Tmp102<I, D> {
    i2c: I,
    delay: D,
}

impl<I, D, E> Tmp102<I, D>
where
    I: Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
{
    pub fn new(i2c: I, delay: D) -> Self {
        Self { i2c, delay }
    }

    fn register_read<const N: usize>(
        &mut self,
        reg: Register,
    ) -> Result<[u8; N], <I as WriteRead>::Error> {
        let mut value = [0u8; N];
        self.i2c.write_read(ADDR, &[reg as u8], &mut value)?;
        Ok(value)
    }

    fn register_write<const N: usize>(
        &mut self,
        reg: Register,
        value: [u8; N],
    ) -> Result<(), <I as Write>::Error> {
        self.i2c.write(ADDR, &[reg as u8, value[0], value[1]])
    }

    pub fn oneshot(&mut self, config: Config) -> Result<i16, E> {
        let config = config | Config::SD | Config::OS;
        self.register_write(Register::Config, config.bits().to_be_bytes())?;
        // A single conversion typically takes 26 ms
        self.delay.delay_ms(30);
        loop {
            let config = Config::from_bytes(self.register_read(Register::Config)?);
            if config.contains(Config::OS) {
                break;
            }
            self.delay.delay_ms(2);
        }

        let temp_reg = i16::from_be_bytes(self.register_read(Register::Temperature)?);
        Ok(temp_reg)
    }
}
