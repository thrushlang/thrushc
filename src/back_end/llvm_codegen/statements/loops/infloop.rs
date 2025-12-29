use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::block;
use crate::back_end::llvm_codegen::codegen::LLVMCodegen;

use crate::back_end::llvm_codegen::helpertypes::traits::LLVMFunctionExtensions;
use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;

use std::path::PathBuf;

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
                    PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    codegen.get_mut_context().get_mut_loop_ctx().pop();

    llvm_builder.position_at_end(exit);
}
