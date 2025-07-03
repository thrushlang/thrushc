use std::fmt::Display;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, IntValue, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, ptrgen, typegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::Type},
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
        ptrgen::compile(context, destination, Some(&Type::Ptr(None))).into_pointer_value();

    let new_size: IntValue = valuegen::compile(context, new_size, None).into_int_value();
    let size: IntValue = valuegen::compile(context, size, None).into_int_value();

    let target_data: &TargetData = context.get_target_data();

    let dest_type: BasicTypeEnum =
        typegen::generate_subtype(llvm_context, destination.get_type_unwrapped());

    let dest_alignment: u32 = target_data.get_preferred_alignment(&dest_type);

    if let Ok(ptr) = llvm_builder.build_memset(dest, dest_alignment, new_size, size) {
        return ptr.into();
    }

    self::codegen_abort("Failed to generate memset builtin call.");
    self::compile_null_ptr(context)
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
