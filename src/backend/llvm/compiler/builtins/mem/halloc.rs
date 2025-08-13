use std::fmt::Display;

use inkwell::{builder::Builder, context::Context, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    core::console::logging::{self, LoggingType},
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alloc: &'ctx Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    llvm_builder
        .build_malloc(typegen::generate_type(llvm_context, alloc), "")
        .unwrap_or_else(|_| {
            self::codegen_abort("Failed to allocate heap memory with halloc builtin.")
        })
        .into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
