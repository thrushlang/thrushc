#![allow(clippy::type_complexity)]

use std::{fmt::Display, rc::Rc};

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, memory, rawgen},
    core::console::logging::{self, LoggingType},
    frontend::types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: &'ctx (
        Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
        Option<Rc<ThrushStatement<'ctx>>>,
    ),
    kind: &ThrushType,
) -> BasicValueEnum<'ctx> {
    match value {
        (Some((name, _)), _) => {
            let ptr: PointerValue = context.get_allocated_symbol(name).raw_load();

            memory::load_anon(context, ptr, kind)
        }
        (_, Some(expr)) => {
            let ptr: PointerValue = rawgen::compile(context, expr, None).into_pointer_value();

            memory::load_anon(context, ptr, kind)
        }
        _ => {
            self::codegen_abort("Invalid load target in expression");
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
