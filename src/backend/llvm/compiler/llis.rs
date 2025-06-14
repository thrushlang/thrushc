use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::rawgen,
    frontend::types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

use super::{context::LLVMCodeGenContext, valuegen};

pub fn compile<'ctx>(
    name: &'ctx str,
    kind: &'ctx ThrushType,
    expr: &'ctx ThrushStatement,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) {
    let value: BasicValueEnum = if kind.is_ptr_type() {
        rawgen::compile(context, expr, Some(kind))
    } else {
        valuegen::compile(context, expr, Some(kind))
    };

    context.alloc_low_level_instruction(name, kind, value);
}
