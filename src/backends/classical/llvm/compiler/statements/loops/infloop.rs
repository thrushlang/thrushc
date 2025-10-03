use std::path::PathBuf;

use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, values::FunctionValue};

use crate::{
    backends::classical::llvm::compiler::{abort, block, codegen::LLVMCodegen},
    frontends::classical::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    if let Ast::Loop { block, span, .. } = stmt {
        let start: BasicBlock = block::append_block(llvm_context, llvm_function);
        let exit: BasicBlock = block::append_block(llvm_context, llvm_function);

        llvm_builder
            .build_unconditional_branch(start)
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    codegen.get_mut_context(),
                    "Failed to compile loop terminator to start!",
                    *span,
                    PathBuf::from(file!()),
                    line!(),
                )
            });

        llvm_builder.position_at_end(start);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_continue_branch(start);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_break_branch(exit);

        codegen.codegen_block(block);

        if codegen
            .get_context()
            .get_last_builder_block()
            .get_terminator()
            .is_none()
        {
            llvm_builder
                .build_unconditional_branch(start)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        codegen.get_mut_context(),
                        "Failed to compile loop body terminator to start!",
                        *span,
                        PathBuf::from(file!()),
                        line!(),
                    )
                });
        }

        codegen.get_mut_context().get_mut_loop_ctx().pop();

        llvm_builder.position_at_end(exit);
    }
}
