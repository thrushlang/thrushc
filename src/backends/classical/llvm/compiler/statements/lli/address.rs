use std::fmt::Display;

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, IntValue, PointerValue},
};

use crate::{
    backends::classical::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, SymbolAllocated},
        ptr, value,
    },
    core::console::logging::{self, LoggingType},
    frontends::classical::{
        types::ast::{Ast, types::AstEitherExpression},
        typesystem::types::Type,
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx AstEitherExpression<'ctx>,
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let indexes: Vec<IntValue> = indexes
        .iter()
        .map(|index| value::compile(context, index, Some(&Type::U32)).into_int_value())
        .collect();

    match source {
        (Some((name, _)), ..) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            symbol
                .gep(context, llvm_context, llvm_builder, &indexes)
                .into()
        }

        (_, Some(expr), ..) => {
            let kind: &Type = expr.get_type_unwrapped();
            let ptr: PointerValue = ptr::compile(context, expr, None).into_pointer_value();

            memory::gep_anon(context, ptr, kind, &indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid address target in expression".to_string());
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
