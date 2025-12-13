use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::generation::cast;
use crate::back_end::llvm_codegen::memory;
use crate::back_end::llvm_codegen::refptr;

use crate::core::diagnostic::span::Span;

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    kind: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = refptr::compile(context, source, None).into_pointer_value();
    let value: BasicValueEnum = memory::load_anon(context, ptr, kind, span);

    cast::try_cast(context, cast_type, kind, value, span)
}
