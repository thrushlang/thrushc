use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::valuegen::ExpressionModificator,
    frontend::types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

use super::{context::LLVMCodeGenContext, valuegen};

pub fn compile<'ctx>(
    name: &'ctx str,
    kind: &'ctx ThrushType,
    expression: &'ctx ThrushStatement,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) {
    let value: BasicValueEnum = valuegen::compile(
        context,
        expression,
        kind,
        ExpressionModificator::new(false, true),
    );

    context.alloc_low_level_instruction(name, value, kind);
}
