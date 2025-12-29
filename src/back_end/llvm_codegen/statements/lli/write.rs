use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::memory;

use crate::core::diagnostic::span::Span;
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
    let ptr: PointerValue = codegen::compile_as_ptr(context, source, None).into_pointer_value();
    let value: BasicValueEnum = codegen::compile(context, write_value, Some(write_type));

    memory::store_anon(context, ptr, value, span);

    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
