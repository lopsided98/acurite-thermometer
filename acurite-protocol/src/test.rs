use super::*;

#[test]
fn test_lfsr_sequence() {
    let seq = lfsr_sequence::<28>();
    assert_eq!(
        [
            0x3e, 0x1f, 0x97, 0xd3, 0xf1, 0xe0, 0x70, 0x38, 0x1c, 0x0e, 0x07, 0x9b, 0xd5, 0xf2,
            0x79, 0xa4, 0x52, 0x29, 0x8c, 0x46, 0x23, 0x89, 0xdc, 0x6e, 0x37, 0x83, 0xd9, 0xf4
        ],
        seq
    );
}

#[test]
fn test_lfsr_hash_zero() {
    let hash = lfsr_hash::<3, 28, 4>(&[0x00, 0x00, 0x00]);
    assert_eq!(0x00, hash);
}

#[test]
fn test_lfsr_hash() {
    let hash = lfsr_hash::<3, 28, 4>(&[0xe6, 0x81, 0x36]);
    assert_eq!(0x13, hash);
}

#[test]
fn test_0606tx_convert_temperature() {
    assert_eq!(1500, tx0606::convert_temperature(0x0960 << 3)); // 150
    assert_eq!(1280, tx0606::convert_temperature(0x0800 << 3)); // 128
    assert_eq!(1279, tx0606::convert_temperature(0x07ff << 3)); // 127.9375
    assert_eq!(3, tx0606::convert_temperature(0x0004 << 3)); // 0.25
    assert_eq!(-3, tx0606::convert_temperature(0x1ffc << 3)); // -0.25
    assert_eq!(-8, tx0606::convert_temperature(0x1ff4 << 3)); // -0.75
    assert_eq!(-254, tx0606::convert_temperature(0x1e6a << 3)); // -25.375
    assert_eq!(-455, tx0606::convert_temperature(0x1d28 << 3)); // -45.5
    assert_eq!(-456, tx0606::convert_temperature(0x1d26 << 3)); // -45.625
    assert_eq!(-458, tx0606::convert_temperature(0x1d24 << 3)); // -45.75
}

#[test]
fn test_0606tx_message() {
    let message = tx0606::generate(0xe6, true, 310);
    assert_eq!([0xe6, 0x81, 0x36, 0x13], message);
}

#[test]
fn test_0606tx_message_low_battery() {
    let message = tx0606::generate(0xe6, false, 310);
    assert_eq!([0xe6, 0x01, 0x36, 0xC6], message);
}
