#![no_std]

use bitvec::view::BitView;

const fn lfsr_sequence<const N: usize>() -> [u8; N] {
    let mut reg: u8 = 0x7C;
    let mut temp_reg: u8;
    let mut sequence = [0u8; N];

    let mut i: usize = 0;
    loop {
        if i >= N {
            break;
        }
        temp_reg = reg & 0x01;
        reg >>= 1;
        reg |= temp_reg << 7;

        if temp_reg != 0 {
            reg ^= 0x18
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
        for (bit_idx, bit) in byte
            .view_bits::<bitvec::order::Msb0>()
            .into_iter()
            .enumerate()
        {
            if *bit {
                hash_reg ^= sequence[byte_idx * 8 + bit_idx + OFFSET]
            }
        }
    }

    hash_reg
}