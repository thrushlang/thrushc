use thrushc_ast::Ast;
use thrushc_ast::traits::AstCodeLocation;
use thrushc_ast::traits::AstStandardExtensions;
use thrushc_span::Span;
use thrushc_typesystem::Type;

use crate::abort;
use crate::block;
use crate::codegen;
use crate::codegen::LLVMCodegen;
use crate::traits::AstLLVMGetType;
use crate::traits::LLVMFunctionExtensions;

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

    let Ast::For {
        local,
        condition,
        actions,
        block,
        span,
        ..
    } = node
    else {
        return;
    };

    let block_span: Span = block.get_span();

    let start: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
    let cond: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
    let body: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
    let exit: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

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
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                })
        });

    llvm_builder.position_at_end(start);

    codegen.codegen_variables(local);

    llvm_builder
        .build_unconditional_branch(cond)
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                codegen.get_mut_context(),
                "Failed to compile for loop start terminator to condition!",
                block.get_span(),
                std::path::PathBuf::from(file!()),
                line!(),
            )
        });

    llvm_builder.position_at_end(cond);

    let condition_type: &Type = condition.llvm_get_type();

    let comparison: IntValue =
        codegen::compile(codegen.get_mut_context(), condition, Some(condition_type))
            .into_int_value();

    llvm_builder
        .build_conditional_branch(comparison, body, exit)
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                codegen.get_mut_context(),
                "Failed to compile for loop comparison to body!",
                condition.get_span(),
                std::path::PathBuf::from(file!()),
                line!(),
            )
        });

    llvm_builder.position_at_end(body);

    codegen
        .get_mut_context()
        .get_mut_loop_ctx()
        .add_continue_branch(cond);

    codegen
        .get_mut_context()
        .get_mut_loop_ctx()
        .add_break_branch(exit);

    if actions.is_before_unary() {
        let _ = codegen::compile(codegen.get_mut_context(), actions, None);
        codegen.codegen_block(block);
    } else {
        codegen.codegen_block(block);
        let _ = codegen::compile(codegen.get_mut_context(), actions, None);
    }

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
                    "Failed to compile for loop body terminator to comparison!",
                    condition.get_span(),
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    codegen.get_mut_context().get_mut_loop_ctx().pop();

    llvm_builder.position_at_end(exit);
}
