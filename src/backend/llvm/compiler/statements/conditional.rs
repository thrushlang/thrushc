use std::fmt::Display;

use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, values::IntValue};

use crate::{
    backend::llvm::compiler::{codegen::LLVMCodegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let abort = |_| {
        self::codegen_abort("Failed to compile conditional statement.");
        unreachable!()
    };

    match codegen.get_mut_context().get_current_fn() {
        Some(function) => {
            if let Ast::If {
                condition,
                block,
                elseif,
                anyway,
                ..
            } = stmt
            {
                let comparison: IntValue =
                    valuegen::compile(codegen.get_mut_context(), condition, Some(&Type::Bool))
                        .into_int_value();

                let then_block: BasicBlock = llvm_context.append_basic_block(function, "if");

                let else_if_cond: BasicBlock = llvm_context.append_basic_block(function, "elseif");

                let else_if_body: BasicBlock =
                    llvm_context.append_basic_block(function, "elseif_body");

                let else_block: BasicBlock = llvm_context.append_basic_block(function, "else");

                let merge_block: BasicBlock = llvm_context.append_basic_block(function, "merge");

                if !elseif.is_empty() {
                    llvm_builder
                        .build_conditional_branch(comparison, then_block, else_if_cond)
                        .unwrap_or_else(abort);
                } else if anyway.is_some() && elseif.is_empty() {
                    llvm_builder
                        .build_conditional_branch(comparison, then_block, else_block)
                        .unwrap_or_else(abort);
                } else {
                    llvm_builder
                        .build_conditional_branch(comparison, then_block, merge_block)
                        .unwrap_or_else(abort);
                }

                llvm_builder.position_at_end(then_block);

                codegen.codegen_code_block(block);

                if !block.has_return() && !block.has_break() && !block.has_continue() {
                    llvm_builder
                        .build_unconditional_branch(merge_block)
                        .unwrap_or_else(abort);
                }

                if !elseif.is_empty() {
                    llvm_builder.position_at_end(else_if_cond);
                } else {
                    llvm_builder.position_at_end(merge_block);
                }

                if !elseif.is_empty() {
                    let elseifs_len: usize = elseif.len();
                    let mut current_block: BasicBlock = else_if_body;

                    elseif.iter().enumerate().for_each(|(idx, ast)| {
                        if let Ast::Elif {
                            condition, block, ..
                        } = ast
                        {
                            let compiled_else_if_cond: IntValue = valuegen::compile(
                                codegen.get_mut_context(),
                                condition,
                                Some(&Type::Bool),
                            )
                            .into_int_value();

                            let elif_body: BasicBlock = current_block;

                            let next_block: BasicBlock = if idx + 1 < elseifs_len {
                                llvm_context.append_basic_block(function, "elseif_body")
                            } else if anyway.is_some() {
                                else_block
                            } else {
                                merge_block
                            };

                            llvm_builder
                                .build_conditional_branch(
                                    compiled_else_if_cond,
                                    elif_body,
                                    next_block,
                                )
                                .unwrap();

                            llvm_builder.position_at_end(elif_body);

                            codegen.codegen_code_block(block);

                            if !block.has_return() && !block.has_break() && !block.has_continue() {
                                let _ = llvm_builder.build_unconditional_branch(merge_block);
                            }

                            if idx + 1 < elseifs_len {
                                llvm_builder.position_at_end(next_block);

                                current_block =
                                    llvm_context.append_basic_block(function, "elseif_body");
                            }
                        }
                    });
                }

                if let Some(otherwise) = anyway {
                    if let Ast::Else { block, .. } = &**otherwise {
                        llvm_builder.position_at_end(else_block);

                        codegen.codegen_code_block(block);

                        if !block.has_return() && !block.has_break() && !block.has_continue() {
                            let _ = llvm_builder.build_unconditional_branch(merge_block);
                        }
                    }
                }

                if !elseif.is_empty() || anyway.is_some() {
                    llvm_builder.position_at_end(merge_block);
                }

                if elseif.is_empty() {
                    let _ = else_if_cond.remove_from_function();
                    let _ = else_if_body.remove_from_function();
                }

                if anyway.is_none() {
                    let _ = else_block.remove_from_function();
                }
            } else {
                self::codegen_abort("Expected conditional to compile.");
            }
        }

        None => {
            self::codegen_abort("No function is currently being compiled.");
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
