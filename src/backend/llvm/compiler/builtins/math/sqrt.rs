use std::fmt::Display;

use inkwell::{
    AddressSpace,
    builder::Builder,
    module::Module,
    types::FunctionType,
    values::{BasicValueEnum, FunctionValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, intrinsics, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: &'ctx ThrushStatement,
) -> BasicValueEnum<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match value.get_type_unwrapped() {
        ThrushType::F32 => {
            let fn_sqrt_type: (&str, FunctionType) =
                intrinsics::math::sqrt::float_instrinsic(context);

            let fn_name: &str = fn_sqrt_type.0;
            let fn_type: FunctionType = fn_sqrt_type.1;

            let llvm_fn_value: FunctionValue =
                if let Some(llvm_fn_value) = llvm_module.get_function(fn_name) {
                    llvm_fn_value
                } else {
                    llvm_module.add_function(fn_name, fn_type, None)
                };

            let value: BasicValueEnum = valuegen::compile(context, value, Some(&ThrushType::F32));

            if let Ok(call) = llvm_builder.build_call(llvm_fn_value, &[value.into()], "") {
                call.try_as_basic_value().unwrap_left()
            } else {
                self::codegen_abort("Could not call built-in sqrt.");
                self::compile_null_ptr(context)
            }
        }

        ThrushType::F64 => {
            let fn_sqrt_type: (&str, FunctionType) =
                intrinsics::math::sqrt::double_instrinsic(context);

            let fn_name: &str = fn_sqrt_type.0;
            let fn_type: FunctionType = fn_sqrt_type.1;

            let llvm_fn_value: FunctionValue =
                if let Some(llvm_fn_value) = llvm_module.get_function(fn_name) {
                    llvm_fn_value
                } else {
                    llvm_module.add_function(fn_name, fn_type, None)
                };

            let value: BasicValueEnum = valuegen::compile(context, value, Some(&ThrushType::F64));

            if let Ok(call) = llvm_builder.build_call(llvm_fn_value, &[value.into()], "") {
                call.try_as_basic_value().unwrap_left()
            } else {
                self::codegen_abort("Could not call built-in sqrt.");
                self::compile_null_ptr(context)
            }
        }

        _ => {
            self::codegen_abort("Could not call built-in sqrt, types differ from range.");
            self::compile_null_ptr(context)
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(
        LoggingType::Bug,
        &format!("CODE GENERATION: '{}'.", message),
    );
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
