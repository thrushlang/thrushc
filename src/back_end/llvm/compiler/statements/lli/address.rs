use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use crate::{
    back_end::llvm::compiler::{
        codegen,
        context::LLVMCodeGenContext,
        memory::{self},
        ptr,
    },
    front_end::{lexer::span::Span, types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let indexes: Vec<IntValue> = indexes
        .iter()
        .map(|index| codegen::compile(context, index, Some(&Type::U32)).into_int_value())
        .collect();

    let span: Span = source.get_span();
    let kind: &Type = source.llvm_get_type(context);
    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();

    memory::gep_anon(context, ptr, kind, &indexes, span).into()
}
