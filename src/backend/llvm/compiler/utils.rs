use std::ops::RangeInclusive;

pub const SHORT_RANGE_OBFUSCATION: RangeInclusive<usize> = 5..=10;
pub const LONG_RANGE_OBFUSCATION: RangeInclusive<usize> = 15..=30;

pub fn generate_random_string(range: RangeInclusive<usize>) -> String {
    let length: usize = fastrand::usize(range);
    let mut random_string: String = String::with_capacity(length);

    for _ in 0..length {
        match fastrand::u8(0..52) {
            n @ 0..=25 => random_string.push((b'A' + n) as char),
            n @ 26..=51 => random_string.push((b'a' + (n - 26)) as char),
            _ => random_string.push('_'),
        }
    }

    random_string
}
