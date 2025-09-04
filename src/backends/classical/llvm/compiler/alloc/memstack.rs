use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::typesystem::types::Type;

use std::fmt::Display;

use inkwell::{types::BasicTypeEnum, values::PointerValue};

#[inline]
pub fn try_alloc_stack<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    kind: &Type,
) -> PointerValue<'ctx> {
    if let Ok(ptr) = context
        .get_llvm_builder()
        .build_alloca(llvm_type, ascii_name)
    {
        return ptr;
    }

    self::codegen_abort(format!(
        "Failed to allocate stack memory for type '{}'.",
        kind
    ));
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
