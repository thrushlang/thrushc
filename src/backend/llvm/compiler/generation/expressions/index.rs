use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::indexes;
use crate::backend::llvm::compiler::memory;
use crate::backend::llvm::compiler::ptr;

use crate::frontend::lexer::span::Span;
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

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
