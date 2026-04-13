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

use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use thrustc_ast::{Ast, traits::AstCodeLocation};
use thrustc_span::Span;
use thrustc_typesystem::{
    Type,
    traits::{InfererTypeExtensions, TypePointerExtensions},
};

use crate::{codegen, context::LLVMCodeGenContext, expressions, memory, traits::AstLLVMGetType};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    index: &'ctx Ast<'ctx>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue =
        codegen::compile_as_ptr_value(context, source, None).into_pointer_value();

    let mut ptr_type: &Type = source.llvm_get_type();
    let infered_inner_type: Type = ptr_type.get_inferer_inner_type();

    let ordered_indexes: Vec<IntValue> = {
        let span: Span = index.get_span();

        let has_inferer_inner_type: bool =
            ptr_type.has_inferer_inner_type() && ptr_type.is_inferer_inner_type_valid();

        if has_inferer_inner_type {
            ptr_type = &infered_inner_type;
        }

        let is_ptr_aggv_type: bool = ptr_type.is_ptr_aggregate_value_like_type();
        let is_ptr_like_type: bool = ptr_type.is_ptr_like_type();

        let indexes: Vec<IntValue> = if is_ptr_aggv_type {
            let base: IntValue = expressions::integer::compile(
                context,
                &Type::U32(span),
                0,
                false,
                index.get_span(),
            );
            let depth: IntValue =
                codegen::compile_as_value(context, index, Some(&Type::U32(span))).into_int_value();

            vec![base, depth]
        } else if is_ptr_like_type {
            let base: IntValue =
                codegen::compile_as_value(context, index, Some(&Type::U64(span))).into_int_value();

            vec![base]
        } else {
            let base: IntValue = expressions::integer::compile(
                context,
                &Type::U32(span),
                0,
                false,
                index.get_span(),
            );
            let depth: IntValue =
                codegen::compile_as_value(context, index, Some(&Type::U32(span))).into_int_value();

            vec![base, depth]
        };

        indexes
    };

    memory::gep_anon(context, ptr, ptr_type, &ordered_indexes, source.get_span()).into()
}
