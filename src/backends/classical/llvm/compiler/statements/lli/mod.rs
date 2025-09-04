use std::fmt::Display;

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    backends::classical::llvm::compiler::{context::LLVMCodeGenContext, valuegen},
    core::console::logging::{self, LoggingType},
    frontends::classical::{types::ast::Ast, typesystem::types::Type},
};

pub mod address;
pub mod alloc;
pub mod load;
pub mod write;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &'ctx str,
    kind: &'ctx Type,
    expr: &'ctx Ast,
) {
    let value: BasicValueEnum = valuegen::compile(context, expr, Some(kind));

    context.new_lli(name, kind, value);
}

pub fn compile_advanced<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Write {
            source,
            write_type,
            write_value,
            ..
        } => self::write::compile(context, source, write_type, write_value),

        Ast::Load { source, kind, .. } => self::load::compile(context, source, kind, cast_type),

        Ast::Address {
            source, indexes, ..
        } => self::address::compile(context, source, indexes),

        Ast::Alloc {
            alloc,
            site_allocation,
            ..
        } => self::alloc::compile(context, alloc, site_allocation),

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
