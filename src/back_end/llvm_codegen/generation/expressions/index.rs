use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::indexes;
use crate::back_end::llvm_codegen::memory;
use crate::back_end::llvm_codegen::refptr;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstLLVMGetType;
use crate::front_end::typesystem::types::Type;

use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    index: &'ctx Ast<'ctx>,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = refptr::compile(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.llvm_get_type(context);

    let ordered_indexes: Vec<IntValue> = indexes::compile(context, index, ptr_type);

    let span: Span = source.get_span();

    memory::gep_anon(context, ptr, ptr_type, &ordered_indexes, span).into()
}
