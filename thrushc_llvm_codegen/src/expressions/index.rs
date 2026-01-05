use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use thrushc_ast::{Ast, traits::AstCodeLocation};
use thrushc_span::Span;
use thrushc_typesystem::{
    Type,
    traits::{TypeIsExtensions, TypePointerExtensions},
};

use crate::{codegen, context::LLVMCodeGenContext, integer, memory, traits::AstLLVMGetType};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    index: &'ctx Ast<'ctx>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = codegen::compile_as_ptr(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.llvm_get_type();

    let ordered_indexes: Vec<IntValue> = {
        let is_ptr_aggv_type: bool = ptr_type.is_ptr_aggregate_value_like_type();
        let is_ptr_like_type: bool = ptr_type.is_ptr_like_type();

        let span: Span = index.get_span();

        let indexes: Vec<IntValue> = if is_ptr_aggv_type {
            let base: IntValue =
                integer::compile(context, &Type::U32(span), 0, false, index.get_span());
            let depth: IntValue =
                codegen::compile(context, index, Some(&Type::U32(span))).into_int_value();

            vec![base, depth]
        } else if is_ptr_like_type {
            let base: IntValue =
                codegen::compile(context, index, Some(&Type::U64(span))).into_int_value();

            vec![base]
        } else {
            let base: IntValue =
                integer::compile(context, &Type::U32(span), 0, false, index.get_span());
            let depth: IntValue =
                codegen::compile(context, index, Some(&Type::U32(span))).into_int_value();

            vec![base, depth]
        };

        indexes
    };

    memory::gep_anon(context, ptr, ptr_type, &ordered_indexes, source.get_span()).into()
}
