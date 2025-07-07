use std::fmt::Display;

use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, values::IntValue};

use crate::{
    backend::llvm::compiler::{codegen::LLVMCodegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    match codegen.get_mut_context().get_current_fn() {
        Some(function) => {
            if let Ast::If {
                cond,
                block,
                elfs,
                otherwise,
                ..
            } = stmt
            {
                let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
                let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

                let if_comparison: IntValue =
                    valuegen::compile(codegen.get_mut_context(), cond, Some(&Type::Bool))
                        .into_int_value();

                let then_block: BasicBlock = llvm_context.append_basic_block(function, "if");

                let else_if_cond: BasicBlock = llvm_context.append_basic_block(function, "elseif");

                let else_if_body: BasicBlock =
                    llvm_context.append_basic_block(function, "elseif_body");

                let else_block: BasicBlock = llvm_context.append_basic_block(function, "else");

                let merge_block: BasicBlock = llvm_context.append_basic_block(function, "merge");

                if !elfs.is_empty() {
                    let _ = llvm_builder.build_conditional_branch(
                        if_comparison,
                        then_block,
                        else_if_cond,
                    );
                } else if otherwise.is_some() && elfs.is_empty() {
                    let _ = llvm_builder.build_conditional_branch(
                        if_comparison,
                        then_block,
                        else_block,
                    );
                } else {
                    let _ = llvm_builder.build_conditional_branch(
                        if_comparison,
                        then_block,
                        merge_block,
                    );
                }

                llvm_builder.position_at_end(then_block);

                codegen.codegen_code_block(block);

                if !block.has_return() && !block.has_break() && !block.has_continue() {
                    let _ = llvm_builder.build_unconditional_branch(merge_block);
                }

                if !elfs.is_empty() {
                    llvm_builder.position_at_end(else_if_cond);
                } else {
                    llvm_builder.position_at_end(merge_block);
                }

                if !elfs.is_empty() {
                    let mut current_block: BasicBlock = else_if_body;

                    for (index, instr) in elfs.iter().enumerate() {
                        if let Ast::Elif { cond, block, .. } = instr {
                            let compiled_else_if_cond: IntValue = valuegen::compile(
                                codegen.get_mut_context(),
                                cond,
                                Some(&Type::Bool),
                            )
                            .into_int_value();

                            let elif_body: BasicBlock = current_block;

                            let next_block: BasicBlock = if index + 1 < elfs.len() {
                                llvm_context.append_basic_block(function, "elseif_body")
                            } else if otherwise.is_some() {
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

                            if index + 1 < elfs.len() {
                                llvm_builder.position_at_end(next_block);

                                current_block =
                                    llvm_context.append_basic_block(function, "elseif_body");
                            }
                        }
                    }
                }

                if let Some(otherwise) = otherwise {
                    if let Ast::Else { block, .. } = &**otherwise {
                        llvm_builder.position_at_end(else_block);

                        codegen.codegen_code_block(block);

                        if !block.has_return() && !block.has_break() && !block.has_continue() {
                            let _ = llvm_builder.build_unconditional_branch(merge_block);
                        }
                    }
                }

                if !elfs.is_empty() || otherwise.is_some() {
                    llvm_builder.position_at_end(merge_block);
                }

                if elfs.is_empty() {
                    let _ = else_if_cond.remove_from_function();
                    let _ = else_if_body.remove_from_function();
                }

                if otherwise.is_none() {
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
