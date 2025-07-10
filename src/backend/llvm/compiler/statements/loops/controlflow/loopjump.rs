use std::fmt::Display;

use inkwell::{basic_block::BasicBlock, builder::Builder};

use crate::{
    backend::llvm::compiler::codegen::LLVMCodegen,
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let abort = |_| {
        self::codegen_abort("Cannot compile loop control flow 'continue'.");
        unreachable!()
    };

    if let Ast::Continue { .. } = stmt {
        let continue_block: BasicBlock = codegen
            .get_context()
            .get_loop_ctx()
            .get_last_continue_branch();

        llvm_builder
            .build_unconditional_branch(continue_block)
            .unwrap_or_else(abort);
    } else {
        self::codegen_abort("Expected 'break' loop control flow to compile.");
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
