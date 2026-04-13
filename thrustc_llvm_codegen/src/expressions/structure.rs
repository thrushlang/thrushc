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

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};
use thrustc_ast::data::ConstructorData;
use thrustc_span::Span;
use thrustc_typesystem::Type;
use thrustc_typesystem::traits::TypeStructExtensions;

use crate::anchor::PointerAnchor;
use crate::context::LLVMCodeGenContext;
use crate::memory::LLVMAllocationSite;
use crate::{codegen, memory, typegeneration};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    data: &'ctx ConstructorData,
    struct_type: &Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    match context.get_pointer_anchor() {
        Some(anchor) if !anchor.is_triggered() => {
            self::compile_with_anchor(context, data, struct_type, span, *anchor)
        }
        _ => self::compile_without_anchor(context, data, struct_type, span),
    }
}

fn compile_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    data: &'ctx ConstructorData,
    struct_type: &Type,
    span: Span,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    context.mark_pointer_anchor();

    let ptr_type: BasicTypeEnum<'_> = typegeneration::generate_type(context, struct_type);
    let ptr: PointerValue<'_> = anchor.get_pointer();

    let fields_types: &[Type] = struct_type.get_struct_fields();

    let fields: Vec<_> = data
        .iter()
        .zip(fields_types)
        .map(|((_, field, _, _), kind)| codegen::compile_as_value(context, field, Some(kind)))
        .collect();

    for (idx, value) in fields.iter().enumerate() {
        if let Ok(field_ptr) = context
            .get_llvm_builder()
            .build_struct_gep(ptr_type, ptr, idx as u32, "")
        {
            memory::store_anon(context, field_ptr, *value, span);
        }
    }

    self::compile_null_ptr(context)
}

fn compile_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    data: &'ctx ConstructorData,
    struct_type: &Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let ptr_type: BasicTypeEnum<'_> = typegeneration::generate_type(context, struct_type);
    let ptr: PointerValue<'_> =
        memory::alloc_anon(context, LLVMAllocationSite::Stack, struct_type, span);

    let fields_types: &[Type] = struct_type.get_struct_fields();

    let fields: Vec<_> = data
        .iter()
        .zip(fields_types)
        .map(|((_, field, _, _), kind)| codegen::compile_as_value(context, field, Some(kind)))
        .collect();

    for (idx, value) in fields.iter().enumerate() {
        if let Ok(field_ptr) = context
            .get_llvm_builder()
            .build_struct_gep(ptr_type, ptr, idx as u32, "")
        {
            memory::store_anon(context, field_ptr, *value, span);
        }
    }

    memory::load_anon(context, ptr, struct_type, span)
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
