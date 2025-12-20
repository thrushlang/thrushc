use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::block;
use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::codegen::LLVMCodegen;

use crate::back_end::llvm_codegen::types::traits::LLVMFunctionExtensions;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::FunctionValue;
use inkwell::values::IntValue;

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen
        .get_context()
        .get_current_llvm_function()
        .get_value();

    if let Ast::For {
        local,
        condition,
        actions,
        block,
        span,
        ..
    } = stmt
    {
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
                            PathBuf::from(file!()),
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
                    "Failed to compile for loop comparison to body!",
                    condition.get_span(),
                    PathBuf::from(file!()),
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
            .get_context()
            .get_last_builder_block()
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
                        PathBuf::from(file!()),
                        line!(),
                    )
                });
        }

        codegen.get_mut_context().get_mut_loop_ctx().pop();

        llvm_builder.position_at_end(exit);
    }
}
