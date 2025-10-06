use std::path::PathBuf;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    values::{FunctionValue, IntValue},
};

use crate::{
    backends::classical::llvm::compiler::{
        abort, block,
        codegen::{self, LLVMCodegen},
    },
    frontends::classical::{types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    if let Ast::While { cond, block, .. } = stmt {
        let condition: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
        let body: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
        let exit: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

        llvm_builder
            .build_unconditional_branch(condition)
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    codegen.get_mut_context(),
                    "Failed to compile while loop terminator to condition!",
                    cond.get_span(),
                    PathBuf::from(file!()),
                    line!(),
                )
            });

        llvm_builder.position_at_end(condition);

        let comparison: IntValue =
            codegen::compile(codegen.get_mut_context(), cond, Some(&Type::Bool)).into_int_value();

        llvm_builder
            .build_conditional_branch(comparison, body, exit)
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    codegen.get_mut_context(),
                    "Failed to compile while loop comparison to body!",
                    cond.get_span(),
                    PathBuf::from(file!()),
                    line!(),
                )
            });

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_continue_branch(condition);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_break_branch(exit);

        llvm_builder.position_at_end(body);

        codegen.codegen_block(block);

        if codegen
            .get_context()
            .get_last_builder_block()
            .get_terminator()
            .is_none()
        {
            llvm_builder
                .build_unconditional_branch(condition)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        codegen.get_mut_context(),
                        "Failed to compile while loop body terminator to comparison!",
                        cond.get_span(),
                        PathBuf::from(file!()),
                        line!(),
                    )
                });
        }

        llvm_builder.position_at_end(exit);

        codegen.get_mut_context().get_mut_loop_ctx().pop();
    }
}
