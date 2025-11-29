use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::generation::cast;
use crate::back_end::llvm::compiler::memory;
use crate::back_end::llvm::compiler::ptr;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

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
