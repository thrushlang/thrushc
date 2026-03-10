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


#[derive(Debug, Clone, Copy)]
pub struct ScoperContext {
    loop_depth: u32,
    inside_function: bool,
}

impl ScoperContext {
    #[inline]
    pub fn new() -> Self {
        ScoperContext {
            loop_depth: 0,
            inside_function: false,
        }
    }
}

impl ScoperContext {
    #[inline]
    pub fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    #[inline]
    pub fn leave_loop(&mut self) {
        self.loop_depth -= 1;
    }

    #[inline]
    pub fn enter_function(&mut self) {
        self.inside_function = true;
    }

    #[inline]
    pub fn leave_function(&mut self) {
        self.inside_function = false;
    }
}

impl ScoperContext {
    #[inline]
    pub fn is_inside_loop(&self) -> bool {
        self.loop_depth > 0
    }

    #[inline]
    pub fn is_inside_function(&self) -> bool {
        self.inside_function
    }
}
