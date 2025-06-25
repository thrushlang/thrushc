#![allow(clippy::type_complexity)]

use std::{fmt::Display, rc::Rc};

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
        rawgen, valuegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::ThrushType},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    address_to: &'ctx (Option<(&'ctx str, Rc<Ast<'ctx>>)>, Option<Rc<Ast<'ctx>>>),
    indexes: &'ctx [Ast],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let indexes: Vec<IntValue> = indexes
        .iter()
        .map(|index| valuegen::compile(context, index, Some(&ThrushType::U32)).into_int_value())
        .collect();

    match address_to {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_allocated_symbol(name);

            symbol.gep(llvm_context, llvm_builder, &indexes).into()
        }
        (_, Some(expr)) => {
            let kind: &ThrushType = expr.get_type_unwrapped();
            let ptr: PointerValue = rawgen::compile(context, expr, None).into_pointer_value();

            memory::gep_anon(context, ptr, kind, &indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid address target in expression".to_string());
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
