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

mod array;
mod casting;
mod constant;
mod dereference;
mod fixedarray;
mod fnref;
mod general;
mod index;
mod inferer;
mod location;
pub mod modificators;
mod pointer;
mod precendece;
mod structure;
pub mod traits;
mod void;

use thrustc_span::Span;

use crate::modificators::FunctionReferenceTypeModificator;
use crate::modificators::StructureTypeModificator;

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone)]
pub enum Type {
    S8(Span),
    S16(Span),
    S32(Span),
    S64(Span),
    SSize(Span),

    // Unsigned Integer Type
    U8(Span),
    U16(Span),
    U32(Span),
    U64(Span),
    U128(Span),
    USize(Span),

    // Floating Point Type
    F32(Span),
    F64(Span),
    F128(Span),
    FX8680(Span),
    FPPC128(Span),

    // Boolean Type
    Bool(Span),

    // Char Type
    Char(Span),

    // Constant Type
    Const(std::boxed::Box<Type>, Span),

    // Ptr Type
    Ptr(Option<std::boxed::Box<Type>>, Span),

    // Struct Type
    Struct(String, std::vec::Vec<Type>, StructureTypeModificator, Span),

    // Fixed FixedArray
    FixedArray(std::boxed::Box<Type>, u32, Span),

    // Array Type
    Array {
        base_type: std::boxed::Box<Type>,
        infered_type: Option<(std::boxed::Box<Type>, usize)>,
        span: Span,
    },

    // Memory Address
    Addr(Span),

    // Function Referece
    Fn(
        std::vec::Vec<Type>,
        std::boxed::Box<Type>,
        FunctionReferenceTypeModificator,
        Span,
    ),

    // Void Type
    Void(Span),

    // Unresolved Type
    Unresolved {
        hint: String,
        span: Span,
    },
}
