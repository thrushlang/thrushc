use std::fmt::Display;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    values::{FunctionValue, IntValue},
};

use crate::{
    backend::llvm::compiler::{codegen::LLVMCodegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    let abort = |_| {
        self::codegen_abort("Cannot compile if conditional statement.");
        unreachable!()
    };

    if let Ast::If {
        condition,
        block,
        elseif,
        anyway,
        ..
    } = stmt
    {
        let then: BasicBlock = llvm_context.append_basic_block(llvm_function, "if");
        let merge: BasicBlock = llvm_context.append_basic_block(llvm_function, "merge");

        let next: BasicBlock = if !elseif.is_empty() {
            llvm_context.append_basic_block(llvm_function, "elseif_cond")
        } else if anyway.is_some() {
            llvm_context.append_basic_block(llvm_function, "else")
        } else {
            merge
        };

        let cond_value: IntValue =
            valuegen::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                .into_int_value();

        llvm_builder
            .build_conditional_branch(cond_value, then, next)
            .unwrap_or_else(abort);

        llvm_builder.position_at_end(then);

        codegen.codegen_block(block);

        if let Some(last_block) = llvm_builder.get_insert_block() {
            if last_block.get_terminator().is_none() {
                llvm_builder
                    .build_unconditional_branch(merge)
                    .unwrap_or_else(abort);
            }
        }

        if !elseif.is_empty() {
            self::compile_elseif(codegen, elseif, next, merge);
        }

        if let Some(else_ast) = anyway {
            self::compile_else(codegen, else_ast, next, merge);
        }

        llvm_builder.position_at_end(merge);
    } else {
        self::codegen_abort("Expected conditional to compile.");
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
        unreachable!()
    };

    for (idx, elseif) in nested_elseif.iter().enumerate() {
        if let Ast::Elif {
            condition, block, ..
        } = elseif
        {
            let is_last: bool = idx == nested_elseif.len().saturating_sub(1);

            llvm_builder.position_at_end(current);

            let then: BasicBlock = llvm_context.append_basic_block(llvm_function, "elseif_body");
            let next: BasicBlock = if is_last {
                merge
            } else {
                llvm_context.append_basic_block(llvm_function, "elseif_cond")
            };

            let cond_value: IntValue =
                valuegen::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                    .into_int_value();

            llvm_builder
                .build_conditional_branch(cond_value, then, next)
                .unwrap_or_else(abort);

            llvm_builder.position_at_end(then);

            codegen.codegen_block(block);

            if let Some(last_block) = llvm_builder.get_insert_block() {
                if last_block.get_terminator().is_none() {
                    llvm_builder
                        .build_unconditional_branch(merge)
                        .unwrap_or_else(abort);
                }
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

        if let Some(last_block) = llvm_builder.get_insert_block() {
            if last_block.get_terminator().is_none() {
                let _ = llvm_builder.build_unconditional_branch(merge);
            }
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
