pub fn generate_random_string(max: usize) -> String {
    let length: usize = fastrand::usize(5..=max);

    let mut random_string: String = String::with_capacity(length);

    for _ in 0..length {
        match fastrand::u8(0..62) {
            n @ 0..=9 => random_string.push((b'0' + n) as char),
            n @ 10..=35 => random_string.push((b'A' + n - 10) as char),
            n @ 36..=61 => random_string.push((b'a' + n - 36) as char),

            _ => random_string.push(b'_' as char),
        }
    }

    random_string
}
