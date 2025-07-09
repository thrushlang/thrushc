use std::fmt::Display;

use inkwell::builder::Builder;

use crate::{
    backend::llvm::compiler::codegen::LLVMCodegen,
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let abort = |_| {
        self::codegen_abort("Cannot compile loop break at code generation time.");
        unreachable!()
    };

    if let Ast::Break { .. } = stmt {
        if let Some(end_loop_block) = codegen.get_context().get_end_loop_block() {
            llvm_builder
                .build_unconditional_branch(end_loop_block)
                .unwrap_or_else(abort);
        } else {
            self::codegen_abort("Loop exit block could not be obtained at code generation time.");
        }
    } else {
        self::codegen_abort("Expected break loop control flow to compile.");
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
