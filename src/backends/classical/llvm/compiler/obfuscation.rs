use std::ops::RangeInclusive;

use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;

pub const SHORT_RANGE_OBFUSCATION: RangeInclusive<usize> = 5..=12;
pub const LONG_RANGE_OBFUSCATION: RangeInclusive<usize> = 10..=30;

#[inline]
#[must_use]
pub fn generate_obfuscation_name(
    context: &LLVMCodeGenContext<'_, '_>,
    range: RangeInclusive<usize>,
) -> String {
    let length: usize = fastrand::usize(range);
    let mut random_string: String = String::with_capacity(length);

    if !context.get_compiler_options().need_obfuscate_ir() {
        return String::new();
    } else {
        random_string.push('.');
    }

    for _ in 0..length {
        let n: u8 = fastrand::u8(0..52);

        let c: char = match n {
            0..=25 => (b'A' + n) as char,
            26..=51 => (b'a' + (n - 26)) as char,
            _ => '_',
        };

        random_string.push(c);
    }

    random_string
}
