use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::typegen;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::typesystem::types::Type;

use std::fmt::Display;

use inkwell::{builder::Builder, context::Context, values::BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alloc: &'ctx Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    llvm_builder
        .build_malloc(typegen::generate(llvm_context, alloc), "")
        .unwrap_or_else(|_| {
            self::codegen_abort("Failed to allocate heap memory with halloc builtin.")
        })
        .into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
