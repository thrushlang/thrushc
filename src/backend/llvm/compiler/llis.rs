use inkwell::values::BasicValueEnum;

use crate::types::frontend::{lexer::types::ThrushType, parser::stmts::stmt::ThrushStatement};

use super::{context::LLVMCodeGenContext, valuegen};

pub fn build<'ctx>(
    name: &'ctx str,
    kind: &'ctx ThrushType,
    expression: &'ctx ThrushStatement,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) {
    let value: BasicValueEnum = valuegen::build(expression, kind, context);
    context.alloc_low_level_instruction(name, value, kind);
}
