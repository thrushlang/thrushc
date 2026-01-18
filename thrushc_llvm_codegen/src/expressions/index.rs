use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use thrushc_ast::{Ast, traits::AstCodeLocation};
use thrushc_span::Span;
use thrushc_typesystem::{
    Type,
    traits::{InfererTypeExtensions, TypeIsExtensions, TypePointerExtensions},
};

use crate::{codegen, context::LLVMCodeGenContext, expressions, memory, traits::AstLLVMGetType};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    index: &'ctx Ast<'ctx>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = codegen::compile_as_ptr(context, source, None).into_pointer_value();

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
                codegen::compile(context, index, Some(&Type::U32(span))).into_int_value();

            vec![base, depth]
        } else if is_ptr_like_type {
            let base: IntValue =
                codegen::compile(context, index, Some(&Type::U64(span))).into_int_value();

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
                codegen::compile(context, index, Some(&Type::U32(span))).into_int_value();

            vec![base, depth]
        };

        indexes
    };

    memory::gep_anon(context, ptr, ptr_type, &ordered_indexes, source.get_span()).into()
}
