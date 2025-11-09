use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::indexes;
use crate::back_end::llvm::compiler::memory;
use crate::back_end::llvm::compiler::ptr;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.llvm_get_type(context);

    let ordered_indexes: Vec<IntValue> = indexes::compile(context, indexes, ptr_type);

    let span: Span = source.get_span();

    memory::gep_anon(context, ptr, ptr_type, &ordered_indexes, span).into()
}
