#![allow(clippy::upper_case_acronyms)]

use super::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::codegen;

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).get_ptr().into(),
        _ => codegen::compile(context, expr, cast_type),
    }
}
