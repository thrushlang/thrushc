use crate::backends::classical::llvm::compiler::typegen;
use crate::backends::classical::llvm::compiler::{self, context::LLVMCodeGenContext};

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::typesystem::types::Type;

use std::fmt::Display;

use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum},
    values::BasicValueEnum,
};

pub fn compile<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    sizeof_type: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate(llvm_context, sizeof_type);

    let sizeof_value: BasicValueEnum = llvm_type
        .size_of()
        .unwrap_or_else(|| {
            self::codegen_abort("Unable to get size of type at executation of the sizeof builtin.")
        })
        .into();

    compiler::generation::cast::try_cast(context, cast, sizeof_type, sizeof_value)
        .unwrap_or(sizeof_value)
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
