use std::fmt::Display;

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
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
    write_type: &'ctx Type,
    write_value: &'ctx Ast,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = value::compile(context, write_value, Some(write_type));

    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            symbol.store(context, value);

            self::compile_null_ptr(context)
        }
        (_, Some(expr)) => {
            let ptr: PointerValue = ptr::compile(context, expr, None).into_pointer_value();

            memory::store_anon(context, ptr, value);

            self::compile_null_ptr(context)
        }
        _ => {
            self::codegen_abort("Invalid write target in expression");
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

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
