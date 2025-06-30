use std::fmt::Display;

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::ThrushType},
};

use super::{context::LLVMCodeGenContext, valuegen};

pub mod address;
pub mod alloc;
pub mod load;
pub mod write;

pub fn new<'ctx>(
    name: &'ctx str,
    kind: &'ctx ThrushType,
    expr: &'ctx Ast,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) {
    let value: BasicValueEnum = valuegen::compile(context, expr, Some(kind));

    context.new_lli(name, kind, value);
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Write {
            write_to,
            write_type,
            write_value,
            ..
        } => self::write::compile(context, write_to, write_type, write_value),

        Ast::Load { value, kind, .. } => self::load::compile(context, value, kind, cast_type),

        Ast::Address {
            address_to,
            indexes,
            ..
        } => self::address::compile(context, address_to, indexes),

        Ast::Alloc {
            type_to_alloc,
            site_allocation,
            ..
        } => self::alloc::compile(context, type_to_alloc, site_allocation),

        _ => {
            self::codegen_abort("Failed to compile low-level instruction. Unknown expression.");
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
