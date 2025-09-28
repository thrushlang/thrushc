#![allow(clippy::if_same_then_else)]

use std::fmt::Display;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    values::{FunctionValue, IntValue},
};

use crate::{
    backends::classical::llvm::compiler::{block, codegen::LLVMCodegen, value},
    core::console::logging::{self, LoggingType},
    frontends::classical::{types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    let abort = |_| {
        self::codegen_abort("Cannot compile if conditional statement.");
    };

    if let Ast::If {
        condition,
        block,
        elseif,
        anyway,
        ..
    } = stmt
    {
        let then: BasicBlock = block::append_block(llvm_context, llvm_function);
        let merge: BasicBlock = block::append_block(llvm_context, llvm_function);

        let next: BasicBlock = if !elseif.is_empty() {
            block::append_block(llvm_context, llvm_function)
        } else if anyway.is_some() {
            block::append_block(llvm_context, llvm_function)
        } else {
            merge
        };

        let cond_value: IntValue =
            value::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                .into_int_value();

        llvm_builder
            .build_conditional_branch(cond_value, then, next)
            .unwrap_or_else(abort);

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
                .unwrap_or_else(abort);
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
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();
    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    let mut current: BasicBlock = first_block;

    let abort = |_| {
        self::codegen_abort("Cannot compile elif conditional statement.");
    };

    for (idx, elseif) in nested_elseif.iter().enumerate() {
        if let Ast::Elif {
            condition, block, ..
        } = elseif
        {
            let is_last: bool = idx == nested_elseif.len().saturating_sub(1);

            llvm_builder.position_at_end(current);

            let then: BasicBlock = block::append_block(llvm_context, llvm_function);

            let next: BasicBlock = if is_last {
                merge
            } else {
                block::append_block(llvm_context, llvm_function)
            };

            let cond_value: IntValue =
                value::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                    .into_int_value();

            llvm_builder
                .build_conditional_branch(cond_value, then, next)
                .unwrap_or_else(abort);

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
                    .unwrap_or_else(abort);
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

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
