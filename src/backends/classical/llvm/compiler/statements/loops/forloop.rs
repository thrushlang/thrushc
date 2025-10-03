use std::path::PathBuf;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
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
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    if let Ast::For {
        local,
        cond,
        actions,
        block,
        span,
        ..
    } = stmt
    {
        let start: BasicBlock = block::append_block(llvm_context, llvm_function);
        let condition: BasicBlock = block::append_block(llvm_context, llvm_function);
        let body: BasicBlock = block::append_block(llvm_context, llvm_function);
        let exit: BasicBlock = block::append_block(llvm_context, llvm_function);

        llvm_builder
            .build_unconditional_branch(start)
            .unwrap_or_else(|_| {
                llvm_builder
                    .build_unconditional_branch(start)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            codegen.get_mut_context(),
                            "Failed to compile of starter!",
                            *span,
                            PathBuf::from(file!()),
                            line!(),
                        )
                    })
            });

        llvm_builder.position_at_end(start);

        codegen.codegen_variables(local);

        llvm_builder
            .build_unconditional_branch(condition)
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    codegen.get_mut_context(),
                    "Failed to compile for loop start terminator to condition!",
                    block.get_span(),
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
                    "Failed to compile for loop comparison to body!",
                    cond.get_span(),
                    PathBuf::from(file!()),
                    line!(),
                )
            });

        llvm_builder.position_at_end(body);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_continue_branch(condition);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_break_branch(exit);

        if actions.is_before_unary() {
            codegen.codegen_block(block);
            let _ = codegen::compile(codegen.get_mut_context(), actions, None);
        } else {
            let _ = codegen::compile(codegen.get_mut_context(), actions, None);
            codegen.codegen_block(block);
        }

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
                        "Failed to compile for loop body terminator to comparison!",
                        cond.get_span(),
                        PathBuf::from(file!()),
                        line!(),
                    )
                });
        }

        codegen.get_mut_context().get_mut_loop_ctx().pop();

        llvm_builder.position_at_end(exit);
    }
}
