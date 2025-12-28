use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::generation::integer;

use crate::core::diagnostic::span::Span;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::typesystem::traits::{TypeIsExtensions, TypePointerExtensions};
use crate::front_end::typesystem::types::Type;

use inkwell::values::IntValue;

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    index: &'ctx Ast<'ctx>,
    expr_type: &'ctx Type,
) -> Vec<IntValue<'ctx>> {
    let is_ptr_aggv_type: bool = expr_type.is_ptr_aggregate_value_like_type();
    let is_ptr_like_type: bool = expr_type.is_ptr_like_type();

    let span: Span = index.get_span();

    let indexes: Vec<IntValue> = if is_ptr_aggv_type {
        let base: IntValue =
            integer::generate_const(context, &Type::U32(span), 0, false, index.get_span());
        let depth: IntValue =
            codegen::compile(context, index, Some(&Type::U32(span))).into_int_value();

        vec![base, depth]
    } else if is_ptr_like_type {
        let base: IntValue =
            codegen::compile(context, index, Some(&Type::U64(span))).into_int_value();

        vec![base]
    } else {
        let base: IntValue =
            integer::generate_const(context, &Type::U32(span), 0, false, index.get_span());
        let depth: IntValue =
            codegen::compile(context, index, Some(&Type::U32(span))).into_int_value();

        vec![base, depth]
    };

    indexes
}
