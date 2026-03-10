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
