use std::fmt::Display;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, IntValue, PointerValue},
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
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let indexes: Vec<IntValue> = indexes
        .iter()
        .map(|index| valuegen::compile(context, index, Some(&Type::U32)).into_int_value())
        .collect();

    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_symbol(name);

            symbol.gep(llvm_context, llvm_builder, &indexes).into()
        }
        (_, Some(expr)) => {
            let kind: &Type = expr.get_type_unwrapped();
            let ptr: PointerValue = ptrgen::compile(context, expr, None).into_pointer_value();

            memory::gep_anon(context, ptr, kind, &indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid address target in expression".to_string());
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
