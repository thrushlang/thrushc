use std::fmt::Display;

use inkwell::{
    AddressSpace,
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backends::classical::llvm::compiler::context::LLVMCodeGenContext,
    core::console::logging::{self, LoggingType},
};

pub fn const_ptr_cast<'ctx>(
    value: BasicValueEnum<'ctx>,
    cast: BasicTypeEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    if value.is_pointer_value() {
        let pointer: PointerValue = value.into_pointer_value();

        return pointer.const_cast(cast.into_pointer_type()).into();
    }

    self::codegen_abort("Cannot cast constant pointer value to non-basic type.");
}

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
