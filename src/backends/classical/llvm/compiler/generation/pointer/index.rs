use std::fmt::Display;

use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::indexes;
use crate::backends::classical::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backends::classical::llvm::compiler::ptr;

use crate::backends::classical::types::LLVMEitherExpression;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use crate::core::console::logging::{self, LoggingType};

use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use inkwell::{builder::Builder, context::Context};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx LLVMEitherExpression<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match source {
        (Some((name, _)), ..) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);
            let symbol_type: &Type = symbol.get_type();

            let ordered_indexes: Vec<IntValue> = indexes::compile(context, indexes, symbol_type);

            symbol
                .gep(context, llvm_context, llvm_builder, &ordered_indexes)
                .into()
        }
        (_, Some(expr), ..) => {
            let expr_ptr: PointerValue = ptr::compile(context, expr, None).into_pointer_value();
            let expr_type: &Type = expr.get_type_unwrapped();

            let ordered_indexes: Vec<IntValue> = indexes::compile(context, indexes, expr_type);

            memory::gep_anon(context, expr_ptr, expr_type, &ordered_indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid index target in expression.");
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message))
}
