use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::ptr;
use crate::backends::classical::llvm::compiler::typegen;
use crate::backends::classical::llvm::compiler::value;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use std::fmt::Display;

use inkwell::{
    builder::Builder,
    context::Context,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, IntValue, PointerValue},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    destination: &'ctx Ast,
    new_size: &'ctx Ast,
    size: &'ctx Ast,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let dest: PointerValue =
        ptr::compile(context, destination, Some(&Type::Ptr(None))).into_pointer_value();

    let new_size: IntValue = value::compile(context, new_size, None).into_int_value();
    let size: IntValue = value::compile(context, size, None).into_int_value();

    let target_data: &TargetData = context.get_target_data();

    let dest_type: BasicTypeEnum =
        typegen::generate_subtype_with_all(llvm_context, destination.get_type_unwrapped());

    let dest_alignment: u32 = target_data.get_preferred_alignment(&dest_type);

    if let Ok(ptr) = llvm_builder.build_memset(dest, dest_alignment, new_size, size) {
        return ptr.into();
    }

    self::codegen_abort("Failed to generate memset builtin call.");
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
