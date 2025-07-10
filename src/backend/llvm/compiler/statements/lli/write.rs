use std::fmt::Display;

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, SymbolAllocated},
        ptrgen, valuegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::{
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
    let value: BasicValueEnum = valuegen::compile(context, write_value, Some(write_type));

    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            symbol.store(context, value);

            self::compile_null_ptr(context)
        }
        (_, Some(expr)) => {
            let ptr: PointerValue = ptrgen::compile(context, expr, None).into_pointer_value();

            memory::store_anon(context, ptr, value);

            self::compile_null_ptr(context)
        }
        _ => {
            self::codegen_abort("Invalid write target in expression");
            self::compile_null_ptr(context)
        }
    }
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
