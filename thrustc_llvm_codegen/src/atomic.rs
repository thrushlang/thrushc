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


use inkwell::{AtomicOrdering, values::InstructionValue};

#[derive(Debug, Clone, Copy)]
pub struct LLVMAtomicModificators {
    pub atomic_volatile: bool,
    pub atomic_ord: Option<AtomicOrdering>,
}

#[inline]
pub fn configure_atomic_modificators<'ctx>(
    instr: InstructionValue<'ctx>,
    modificators: LLVMAtomicModificators,
) {
    if modificators.atomic_volatile {
        let _ = instr.set_volatile(true);
    }

    if let Some(ordering) = modificators.atomic_ord {
        let _ = instr.set_atomic_ordering(ordering);
    }
}
