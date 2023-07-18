#![no_std]

#[cfg(test)]
mod test;

const fn lfsr_sequence<const N: usize>() -> [u8; N] {
    let mut reg: u8 = 0x7C;
    let mut sequence = [0u8; N];

    let mut i: usize = 0;
    loop {
        if i >= N {
            break;
        }
        reg = reg.rotate_right(1);
        if reg & (1 << 7) != 0 {
            reg ^= 1 << 3 | 1 << 4;
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
        let mut byte = *byte;
        for bit_idx in 0..8 {
            let bit = byte & 0x80 != 0;
            if bit {
                hash_reg ^= sequence[byte_idx * 8 + bit_idx + OFFSET]
            }
            byte <<= 1;
        }
    }

    hash_reg
}

/// Support for the Acurite 00606TX temperature sensor.
pub mod tx0606 {

    /// Convert a left justified, 9.4 fixed point temperature to the decimal format used by the 0606TX. This is the format used by sensors such as the TMP102.
    pub const fn convert_temperature(temp_reg: i16) -> i16 {
        let temp_whole = temp_reg >> 7;
        // Binary fractional part, out of 16
        let temp_frac_bin = ((temp_reg & 0x7f) >> 3) as u8;
        // Convert to decimal fraction, with proper rounding
        let temp_frac = if temp_whole >= 0 {
            match temp_frac_bin {
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
            }
        } else {
            match temp_frac_bin {
                0 => 0,  // 0
                1 => 1,  // 0.0625
                2 => 1,  // 0.125
                3 => 2,  // 0.1875
                4 => 2,  // 0.25
                5 => 3,  // 0.3125
                6 => 4,  // 0.375
                7 => 4,  // 0.4375
                8 => 5,  // 0.5
                9 => 6,  // 0.5625
                10 => 6, // 0.625
                11 => 7, // 0.6875
                12 => 7, // 0.75
                13 => 8, // 0.8125
                14 => 9, // 0.875
                15 => 9, // 0.9375
                _ => unreachable!(),
            }
        };

        temp_whole * 10 + temp_frac as i16
    }

    pub fn generate(id: u8, battery_ok: bool, temperature: i16) -> [u8; 4] {
        let status = if battery_ok { 0b1000 } else { 0b0000 };
        let body: [u8; 3] = [
            id,
            status << 4 | ((temperature >> 8) as u8 & 0x0f),
            (temperature & 0xff) as u8,
        ];
        let hash = super::lfsr_hash::<3, 28, 4>(&body);
        [body[0], body[1], body[2], hash]
    }
}
