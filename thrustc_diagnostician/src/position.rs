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

#[derive(Debug)]
pub struct CodePosition {
    line: usize,
    start: usize,
    end: usize,
}

impl CodePosition {
    #[inline]
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Self { line, start, end }
    }
}

impl CodePosition {
    #[inline]
    pub fn get_line(&self) -> usize {
        self.line
    }

    #[inline]
    pub fn get_start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn get_end(&self) -> usize {
        self.end
    }
}
