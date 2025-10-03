use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::{codegen, memory, typegen};

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use std::fmt::Display;

use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use inkwell::{builder::Builder, values::BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = codegen::compile(context, source, None);

    if source.is_allocated() {
        self::compile_ptr_property(context, source, indexes)
    } else {
        self::compile_value_property(context, value, indexes)
    }
}

fn compile_value_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let mut last_value: BasicValueEnum = llvm_builder
        .build_extract_value(value.into_struct_value(), indexes[0].1, "")
        .unwrap_or_else(|_| {
            self::codegen_abort(format!(
                "Failed to access property at index '{}' for structure value.",
                indexes[0].1
            ));
        });

    for index in indexes.iter().skip(1) {
        if value.is_struct_value() {
            if let Ok(new_value) =
                llvm_builder.build_extract_value(value.into_struct_value(), index.1, "")
            {
                last_value = new_value;
            }
        }
    }

    last_value
}

fn compile_ptr_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let ptr: PointerValue = codegen::compile(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.get_type_unwrapped();

    let mut property: PointerValue = memory::gep_struct_anon(context, ptr, ptr_type, indexes[0].1);

    for idx in indexes.iter().skip(1) {
        let idx_type: BasicTypeEnum = typegen::generate(llvm_context, &idx.0);

        match llvm_builder.build_struct_gep(idx_type, property, idx.1, "") {
            Ok(new_ptr) => property = new_ptr,
            Err(_) => {
                self::codegen_abort(format!(
                    "Failed to access property at index '{}' for a expression.",
                    idx.1
                ));
            }
        }
    }

    property.into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
