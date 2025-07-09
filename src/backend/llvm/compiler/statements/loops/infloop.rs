use std::fmt::Display;

use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context};

use crate::{
    backend::llvm::compiler::codegen::LLVMCodegen,
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let abort = |_| {
        self::codegen_abort("Cannot compile loop at code generation time.");
        unreachable!()
    };

    let abort_intern = || {
        self::codegen_abort("Cannot compile loop at code generation time.");
        unreachable!()
    };

    match codegen.get_mut_context().get_current_fn() {
        Some(function) => {
            if let Ast::Loop { block, .. } = stmt {
                let start_loop_block: BasicBlock =
                    llvm_context.append_basic_block(function, "loop");

                llvm_builder
                    .build_unconditional_branch(start_loop_block)
                    .unwrap_or_else(abort);

                llvm_builder.position_at_end(start_loop_block);

                let exit_loop_block: BasicBlock =
                    llvm_context.append_basic_block(function, "loop_exit");

                if block.has_break() || block.has_return() {
                    codegen
                        .get_mut_context()
                        .set_end_loop_block(exit_loop_block);
                }

                if block.has_continue() {
                    codegen
                        .get_mut_context()
                        .set_begin_loop_block(start_loop_block);
                }

                codegen.codegen_code_block(block);

                if !block.has_return() && !block.has_break() && !block.has_continue() {
                    let _ = exit_loop_block.remove_from_function();
                    llvm_builder
                        .build_unconditional_branch(
                            function.get_last_basic_block().unwrap_or_else(abort_intern),
                        )
                        .unwrap_or_else(abort);
                } else {
                    llvm_builder.position_at_end(exit_loop_block);
                }
            } else {
                self::codegen_abort("Expected loop to compile.");
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
