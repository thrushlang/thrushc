use crate::back_end::llvm::compiler::codegen;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::memory;
use crate::back_end::llvm::compiler::ptr;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;

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
