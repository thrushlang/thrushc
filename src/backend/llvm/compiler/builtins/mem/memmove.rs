use std::fmt::Display;

use inkwell::{
    builder::Builder,
    context::Context,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, IntValue, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, ptrgen, typegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::{types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast,
    destination: &'ctx Ast,
    size: &'ctx Ast,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let src: PointerValue =
        ptrgen::compile(context, source, Some(&Type::Ptr(None))).into_pointer_value();

    let dest: PointerValue =
        ptrgen::compile(context, destination, Some(&Type::Ptr(None))).into_pointer_value();

    let size: IntValue = valuegen::compile(context, size, None).into_int_value();

    let target_data: &TargetData = context.get_target_data();

    let src_type: BasicTypeEnum =
        typegen::generate_subtype_with_all(llvm_context, source.get_type_unwrapped());

    let dest_type: BasicTypeEnum =
        typegen::generate_subtype_with_all(llvm_context, destination.get_type_unwrapped());

    let src_alignment: u32 = target_data.get_preferred_alignment(&src_type);
    let dest_alignment: u32 = target_data.get_preferred_alignment(&dest_type);

    if let Ok(ptr) = llvm_builder.build_memmove(dest, dest_alignment, src, src_alignment, size) {
        return ptr.into();
    }

    self::codegen_abort("Failed to generate memmove builtin call.");
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
