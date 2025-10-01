use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backends::classical::llvm::compiler::ptr;
use crate::backends::classical::llvm::compiler::typegen;

use crate::frontends::classical::types::ast::types::AstEitherExpression;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::{builder::Builder, context::Context};

use std::fmt::Display;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,

    source: &'ctx AstEitherExpression<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            if !symbol.is_pointer() {
                self::codegen_abort(format!(
                    "Symbol '{}' is not a pointer for property access.",
                    name
                ));
            }

            let mut ptr: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, indexes[0].1);

            for index in indexes.iter().skip(1) {
                let index_type: BasicTypeEnum = typegen::generate(llvm_context, &index.0);

                match llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
                    Ok(new_ptr) => ptr = new_ptr,
                    Err(_) => {
                        self::codegen_abort(format!(
                            "Failed to access property at index '{}' for '{}'.",
                            index.1, name
                        ));
                    }
                }
            }

            ptr.into()
        }

        (_, Some(expr)) => {
            let kind: &Type = expr.get_type_unwrapped();
            let ptr: PointerValue = ptr::compile(context, expr, None).into_pointer_value();

            let mut ptr: PointerValue = memory::get_struct_anon(context, ptr, kind, indexes[0].1);

            for index in indexes.iter().skip(1) {
                let index_type: BasicTypeEnum = typegen::generate(llvm_context, &index.0);

                match llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
                    Ok(new_ptr) => ptr = new_ptr,
                    Err(_) => {
                        self::codegen_abort(format!(
                            "Failed to access property at index '{}' for a expression.",
                            index.1
                        ));
                    }
                }
            }

            ptr.into()
        }

        _ => {
            self::codegen_abort("Unable to compile property access.");
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message))
}
