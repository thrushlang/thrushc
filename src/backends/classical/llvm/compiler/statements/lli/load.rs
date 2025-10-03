use inkwell::values::{BasicValueEnum, PointerValue};

use crate::{
    backends::classical::llvm::compiler::{self, context::LLVMCodeGenContext, memory, ptr},
    frontends::classical::{lexer::span::Span, types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    kind: &Type,
    span: Span,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();
    let value: BasicValueEnum = memory::load_anon(context, ptr, kind, span);

    compiler::generation::cast::try_cast(context, cast, kind, value).unwrap_or(value)
}
