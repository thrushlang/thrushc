use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::block;
use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::codegen::LLVMCodegen;

use crate::back_end::llvm_codegen::helpertypes::traits::LLVMFunctionExtensions;
use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::FunctionValue;
use inkwell::values::IntValue;

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, node: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen
        .get_mut_context()
        .get_current_llvm_function(node.get_span())
        .get_value();

    let Ast::While {
        condition, block, ..
    } = node
    else {
        return;
    };

    let block_span: Span = block.get_span();

    let cond: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
    let body: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
    let exit: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

    llvm_builder
        .build_unconditional_branch(cond)
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                codegen.get_mut_context(),
                "Failed to compile while loop terminator to condition!",
                condition.get_span(),
                PathBuf::from(file!()),
                line!(),
            )
        });

    llvm_builder.position_at_end(cond);

    let comparison: IntValue = codegen::compile(
        codegen.get_mut_context(),
        condition,
        Some(&Type::Bool(condition.get_span())),
    )
    .into_int_value();

    llvm_builder
        .build_conditional_branch(comparison, body, exit)
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                codegen.get_mut_context(),
                "Failed to compile while loop comparison to body!",
                condition.get_span(),
                PathBuf::from(file!()),
                line!(),
            )
        });

    codegen
        .get_mut_context()
        .get_mut_loop_ctx()
        .add_continue_branch(cond);

    codegen
        .get_mut_context()
        .get_mut_loop_ctx()
        .add_break_branch(exit);

    llvm_builder.position_at_end(body);

    codegen.codegen_block(block);

    if codegen
        .get_mut_context()
        .get_last_builder_block(block_span)
        .get_terminator()
        .is_none()
    {
        llvm_builder
            .build_unconditional_branch(cond)
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    codegen.get_mut_context(),
                    "Failed to compile while loop body terminator to comparison!",
                    condition.get_span(),
                    PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    llvm_builder.position_at_end(exit);

    codegen.get_mut_context().get_mut_loop_ctx().pop();
}
