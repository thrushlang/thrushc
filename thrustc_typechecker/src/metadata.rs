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
pub struct TypeCheckerExpressionMetadata {
    is_literal: bool,
}

impl TypeCheckerExpressionMetadata {
    #[inline]
    pub fn new(is_literal: bool) -> Self {
        Self { is_literal }
    }
}

impl TypeCheckerExpressionMetadata {
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.is_literal
    }
}
