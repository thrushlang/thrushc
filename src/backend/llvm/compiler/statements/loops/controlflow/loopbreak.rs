use std::fmt::Display;

use inkwell::builder::Builder;

use crate::{
    backend::llvm::compiler::codegen::LLVMCodegen,
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    if let Ast::Break { .. } = stmt {
        let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

        if let Some(end_loop_block) = codegen.get_context().get_end_loop_block() {
            let _ = llvm_builder.build_unconditional_branch(end_loop_block);
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
