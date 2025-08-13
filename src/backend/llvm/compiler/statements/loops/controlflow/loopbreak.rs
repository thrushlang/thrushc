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
        self::codegen_abort("Cannot compile loop control flow 'break'.");
    };

    if let Ast::Break { .. } = stmt {
        let break_block: BasicBlock = codegen.get_context().get_loop_ctx().get_last_break_branch();

        llvm_builder
            .build_unconditional_branch(break_block)
            .unwrap_or_else(abort);
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
