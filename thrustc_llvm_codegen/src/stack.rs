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

use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use thrustc_attributes::traits::ThrustAttributesExtensions;
use thrustc_attributes::{ThrustAttribute, ThrustAttributes};
use thrustc_span::Span;
use thrustc_typesystem::Type;

use std::path::PathBuf;

use crate::context::LLVMCodeGenContext;
use crate::{abort, heap, typegeneration};

pub fn local_variable<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    kind: &Type,
    attributes: &ThrustAttributes,
    span: Span,
) -> PointerValue<'ctx> {
    let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, kind);

    let name: String = format!("local.{}", ascii_name);

    context.mark_dbg_location(span);

    if attributes.has_heap_attr() {
        heap::try_alloc_at_heap(context, llvm_type, &name, attributes, span)
    } else {
        self::try_alloc_at_stack(context, llvm_type, &name, attributes, span)
    }
}

#[inline]
fn try_alloc_at_stack<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    attributes: &ThrustAttributes,
    span: Span,
) -> PointerValue<'ctx> {
    if let Ok(ptr) = context
        .get_llvm_builder()
        .build_alloca(llvm_type, ascii_name)
    {
        if let Some(align_attr) =
            attributes.get_attr(thrustc_attributes::ThrustAttributeComparator::Align)
        {
            if let Some(instruction) = ptr.as_instruction() {
                if let ThrustAttribute::Align(value, ..) = align_attr {
                    let _ = instruction.set_alignment(value.try_into().unwrap_or(u32::MAX));
                }
            }
        }

        return ptr;
    }

    abort::abort_codegen(
        context,
        "Failed to allocate at stack!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}
