/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/


use crate::context::LLVMCodeGenContext;

pub const SHORT_RANGE_OBFUSCATION: std::ops::RangeInclusive<usize> = 5..=12;
pub const LONG_RANGE_OBFUSCATION: std::ops::RangeInclusive<usize> = 10..=30;

#[inline]
#[must_use]
pub fn generate_string(
    context: &LLVMCodeGenContext<'_, '_>,
    range: std::ops::RangeInclusive<usize>,
) -> String {
    if !context.get_compiler_options().need_obfuscate_ir() {
        String::new()
    } else {
        let length: usize = fastrand::usize(range);
        let mut random_string: String = String::with_capacity(length);

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
}
