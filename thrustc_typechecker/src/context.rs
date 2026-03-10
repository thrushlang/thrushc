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


use thrustc_span::Span;
use thrustc_typesystem::Type;

#[derive(Debug)]
pub struct TypeCheckerTypeContext<'type_checker> {
    current_function_type: Option<(&'type_checker Type, Span)>,
}

impl<'type_checker> TypeCheckerTypeContext<'type_checker> {
    #[inline]
    pub fn new() -> Self {
        Self {
            current_function_type: None,
        }
    }
}

impl<'type_checker> TypeCheckerTypeContext<'type_checker> {
    #[inline]
    pub fn set_current_function_type(&mut self, function_type: (&'type_checker Type, Span)) {
        self.current_function_type = Some(function_type);
    }

    #[inline]
    pub fn unset_current_function_type(&mut self) {
        self.current_function_type = None;
    }
}

impl<'type_checker> TypeCheckerTypeContext<'type_checker> {
    pub fn get_current_function_type(&self) -> Option<(&'type_checker Type, Span)> {
        self.current_function_type
    }
}
