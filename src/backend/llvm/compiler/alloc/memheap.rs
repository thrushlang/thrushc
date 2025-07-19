use std::fmt::Display;

use inkwell::{types::BasicTypeEnum, values::PointerValue};

use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext,
    core::console::logging::{self, LoggingType},
    frontend::typesystem::types::Type,
};

#[inline]
pub fn try_alloc_heap<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    kind: &Type,
) -> PointerValue<'ctx> {
    match context
        .get_llvm_builder()
        .build_malloc(llvm_type, ascii_name)
    {
        Ok(ptr) => ptr,
        Err(_) => {
            self::codegen_abort(format!(
                "Failed to allocate heap memory for type '{}'.",
                kind
            ));

            unreachable!()
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
