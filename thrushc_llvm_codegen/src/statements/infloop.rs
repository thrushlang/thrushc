use thrushc_ast::Ast;
use thrushc_ast::traits::AstCodeLocation;
use thrushc_span::Span;

use crate::abort;
use crate::block;
use crate::codegen::LLVMCodegen;
use crate::traits::LLVMFunctionExtensions;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::FunctionValue;

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, node: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen
        .get_mut_context()
        .get_current_llvm_function(node.get_span())
        .get_value();

    let Ast::Loop { block, span, .. } = node else {
        return;
    };

    let block_span: Span = block.get_span();

    let start: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
    let exit: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

    llvm_builder
        .build_unconditional_branch(start)
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                codegen.get_mut_context(),
                "Failed to compile loop terminator to start!",
                *span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        });

    llvm_builder.position_at_end(start);

    if codegen.get_context().get_loop_ctx().get_all_branch_depth() == 0 {
        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .set_breakall_branch(exit);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .set_continueall_branch(start);
    }

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
        .get_mut_context()
        .get_last_builder_block(block_span)
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
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    codegen.get_mut_context().get_mut_loop_ctx().pop();

    if codegen.get_context().get_loop_ctx().get_branch_depth() == 0 {
        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .pop_superior_branchers();
    }

    llvm_builder.position_at_end(exit);
}
