use std::fmt::Display;

use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, values::FunctionValue};

use crate::{
    backends::classical::llvm::compiler::codegen::LLVMCodegen,
    core::console::logging::{self, LoggingType},
    frontends::classical::types::ast::Ast,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let abort = |_| {
        self::codegen_abort("Cannot compile loop at code generation time.");
        unreachable!()
    };

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    if let Ast::Loop { block, .. } = stmt {
        let start: BasicBlock = llvm_context.append_basic_block(llvm_function, "loop");
        let exit: BasicBlock = llvm_context.append_basic_block(llvm_function, "loop_exit");

        llvm_builder
            .build_unconditional_branch(start)
            .unwrap_or_else(abort);

        llvm_builder.position_at_end(start);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_continue_branch(start);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_break_branch(exit);

        codegen.codegen_block(block);

        if codegen
            .get_context()
            .get_last_builder_block()
            .get_terminator()
            .is_none()
        {
            let _ = llvm_builder.build_unconditional_branch(start);
        }

        codegen.get_mut_context().get_mut_loop_ctx().pop();

        llvm_builder.position_at_end(exit);
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
