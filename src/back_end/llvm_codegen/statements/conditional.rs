#![allow(clippy::if_same_then_else)]

use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::block;
use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::codegen::LLVMCodegen;

use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::FunctionValue;
use inkwell::values::IntValue;

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    if let Ast::If {
        condition,
        block,
        elseif,
        anyway,
        ..
    } = stmt
    {
        let then: BasicBlock = block::append_block(codegen.get_context(), llvm_function);
        let merge: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

        let next: BasicBlock = if !elseif.is_empty() {
            block::append_block(codegen.get_context(), llvm_function)
        } else if anyway.is_some() {
            block::append_block(codegen.get_context(), llvm_function)
        } else {
            merge
        };

        let cond_value: IntValue =
            codegen::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                .into_int_value();

        llvm_builder
            .build_conditional_branch(cond_value, then, next)
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    codegen.get_mut_context(),
                    "Failed to if comparison!",
                    condition.get_span(),
                    PathBuf::from(file!()),
                    line!(),
                )
            });

        llvm_builder.position_at_end(then);

        codegen.codegen_block(block);

        if codegen
            .get_context()
            .get_last_builder_block()
            .get_terminator()
            .is_none()
        {
            llvm_builder
                .build_unconditional_branch(merge)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        codegen.get_mut_context(),
                        "Failed to if terminator!",
                        block.get_span(),
                        PathBuf::from(file!()),
                        line!(),
                    )
                });
        }

        if !elseif.is_empty() {
            self::compile_elseif(codegen, elseif, next, merge);
        }

        if let Some(else_ast) = anyway {
            self::compile_else(codegen, else_ast, next, merge);
        }

        llvm_builder.position_at_end(merge);
    }
}

fn compile_elseif<'ctx>(
    codegen: &mut LLVMCodegen<'_, 'ctx>,
    nested_elseif: &'ctx [Ast<'ctx>],
    first_block: BasicBlock<'ctx>,
    merge: BasicBlock<'ctx>,
) {
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();
    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    let mut current: BasicBlock = first_block;

    for (idx, elseif) in nested_elseif.iter().enumerate() {
        if let Ast::Elif {
            condition, block, ..
        } = elseif
        {
            let is_last: bool = idx == nested_elseif.len().saturating_sub(1);

            llvm_builder.position_at_end(current);

            let then: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

            let next: BasicBlock = if is_last {
                merge
            } else {
                block::append_block(codegen.get_context(), llvm_function)
            };

            let cond_value: IntValue =
                codegen::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                    .into_int_value();

            llvm_builder
                .build_conditional_branch(cond_value, then, next)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        codegen.get_mut_context(),
                        "Failed to elif comparison!",
                        condition.get_span(),
                        PathBuf::from(file!()),
                        line!(),
                    )
                });

            llvm_builder.position_at_end(then);

            codegen.codegen_block(block);

            if codegen
                .get_context()
                .get_last_builder_block()
                .get_terminator()
                .is_none()
            {
                llvm_builder
                    .build_unconditional_branch(merge)
                    .unwrap_or_else(|_| {
                        abort::abort_codegen(
                            codegen.get_mut_context(),
                            "Failed to elif terminator!",
                            block.get_span(),
                            PathBuf::from(file!()),
                            line!(),
                        )
                    });
            }

            current = next;
        }
    }

    llvm_builder.position_at_end(current);
}

pub fn compile_else<'ctx>(
    codegen: &mut LLVMCodegen<'_, 'ctx>,
    anyway: &'ctx Ast<'ctx>,
    next: BasicBlock<'ctx>,
    merge: BasicBlock<'ctx>,
) {
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    if let Ast::Else { block, .. } = anyway {
        llvm_builder.position_at_end(next);

        codegen.codegen_block(block);

        if codegen
            .get_context()
            .get_last_builder_block()
            .get_terminator()
            .is_none()
        {
            let _ = llvm_builder.build_unconditional_branch(merge);
        }
    }
}
