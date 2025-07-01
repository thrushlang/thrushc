use std::fmt::Display;

use inkwell::builder::Builder;

use crate::{
    backend::llvm::compiler::codegen::LLVMCodegen,
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    if let Ast::Continue { .. } = stmt {
        let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

        if let Some(begin_loop_block) = codegen.get_context().get_begin_loop_block() {
            let _ = llvm_builder.build_unconditional_branch(begin_loop_block);
        } else {
            self::codegen_abort("Loop start block could not be obtained at code generation time.");
        }
    } else {
        self::codegen_abort("Expected break loop control flow to compile.");
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
