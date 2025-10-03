#![allow(clippy::upper_case_acronyms)]

use super::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::codegen;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::values::BasicValueEnum;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Reference { name, .. } => context.get_table().get_symbol(name).get_ptr().into(),
        _ => codegen::compile(context, expr, cast),
    }
}
