use crate::backends::classical::llvm::compiler::memory::SymbolAllocated;
use crate::backends::classical::llvm::compiler::value;
use crate::backends::classical::llvm::compiler::{self, context::LLVMCodeGenContext};

use crate::frontends::classical::types::ast::types::AstEitherExpression;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use std::fmt::Display;

use inkwell::{builder::Builder, values::BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx AstEitherExpression<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    match source {
        (Some((name, _)), ..) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            if symbol.is_pointer() {
                compiler::generation::pointer::property::compile(context, source, indexes)
            } else {
                self::compile_extract_value_property(context, symbol.load(context), indexes)
            }
        }
        (None, Some(expr), ..) => {
            let value: BasicValueEnum = value::compile(context, expr, None);

            if value.is_pointer_value() {
                compiler::generation::pointer::property::compile(context, source, indexes)
            } else {
                self::compile_extract_value_property(context, value, indexes)
            }
        }
        _ => {
            self::codegen_abort("Unable to get a value of an structure at memory manipulation.");
        }
    }
}

fn compile_extract_value_property<'ctx>(
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

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
