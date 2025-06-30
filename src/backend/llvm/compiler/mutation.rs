use std::fmt::Display;

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, SymbolAllocated},
        ptrgen, valuegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::ThrushType},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Mut { source, value, .. } => match source {
            (Some(any_reference), None) => {
                let reference_name: &str = any_reference.0;
                let reference: &Ast = &any_reference.1;

                let cast_type: &ThrushType = reference.get_type_unwrapped();

                let symbol: SymbolAllocated = context.get_symbol(reference_name);

                let value: BasicValueEnum = valuegen::compile(context, value, Some(cast_type));

                symbol.store(context, value);

                self::compile_null_ptr(context)
            }

            (None, Some(expr)) => {
                let cast_type: &ThrushType = expr.get_type_unwrapped();

                let ptr: BasicValueEnum = ptrgen::compile(context, expr, None);
                let value: BasicValueEnum = valuegen::compile(context, value, Some(cast_type));

                memory::store_anon(context, ptr.into_pointer_value(), value);

                self::compile_null_ptr(context)
            }

            (None, None) => {
                self::codegen_abort("A mutation must have a source.");
                self::compile_null_ptr(context)
            }

            _ => {
                self::codegen_abort("The source of a mutation could not be obtained.");
                self::compile_null_ptr(context)
            }
        },

        _ => {
            self::codegen_abort("A mutation cannot be executed.");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
