use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{
        codegen,
        context::LLVMCodeGenContext,
        memory::{self},
        ptr,
    },
    frontend::{lexer::span::Span, types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    write_type: &'ctx Type,
    write_value: &'ctx Ast,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();
    let value: BasicValueEnum = codegen::compile(context, write_value, Some(write_type));

    memory::store_anon(context, ptr, value, span);

    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
