use std::fmt::Display;

use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, values::IntValue};

use crate::{
    backend::llvm::compiler::{codegen::LLVMCodegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let forloop_abort = |_| {
        self::codegen_abort("Cannot compile for loop at code generation time.");
        unreachable!()
    };

    match codegen.get_mut_context().get_current_fn() {
        Some(function) => {
            if let Ast::For {
                local,
                cond,
                actions,
                block,
                ..
            } = stmt
            {
                let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
                let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

                codegen.codegen_variables(local);

                let start_block: BasicBlock = llvm_context.append_basic_block(function, "for");

                let _ = llvm_builder.build_unconditional_branch(start_block);

                llvm_builder.position_at_end(start_block);

                let condition: IntValue =
                    valuegen::compile(codegen.get_mut_context(), cond, Some(&Type::Bool))
                        .into_int_value();

                let then_block: BasicBlock = llvm_context.append_basic_block(function, "for_body");
                let exit_block: BasicBlock = llvm_context.append_basic_block(function, "for_exit");

                llvm_builder
                    .build_conditional_branch(condition, then_block, exit_block)
                    .unwrap_or_else(forloop_abort);

                if block.has_break() || block.has_return() {
                    codegen.get_mut_context().set_end_loop_block(exit_block);
                }

                if block.has_continue() {
                    codegen.get_mut_context().set_begin_loop_block(start_block);
                }

                llvm_builder.position_at_end(then_block);

                if actions.is_before_unary() {
                    codegen.codegen_code_block(block);
                    let _ = valuegen::compile(codegen.get_mut_context(), actions, None);
                } else {
                    let _ = valuegen::compile(codegen.get_mut_context(), actions, None);
                    codegen.codegen_code_block(block);
                }

                if !block.has_break() || !block.has_return() || !block.has_continue() {
                    let _ = llvm_builder.build_unconditional_branch(start_block);
                }

                llvm_builder.position_at_end(exit_block);
            } else {
                self::codegen_abort("Expected for loop to compile.");
            }
        }

        None => {
            self::codegen_abort("The function being compiled could not be obtained.");
        }
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
