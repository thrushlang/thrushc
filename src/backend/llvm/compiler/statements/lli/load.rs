use inkwell::values::{BasicValueEnum, PointerValue};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, generation::cast, memory, ptr},
    frontend::{lexer::span::Span, types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    kind: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();
    let value: BasicValueEnum = memory::load_anon(context, ptr, kind, span);

    cast::try_cast(context, cast_type, kind, value, span).unwrap_or(value)
}
