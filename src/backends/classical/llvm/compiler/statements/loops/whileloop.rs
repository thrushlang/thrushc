use std::fmt::Display;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    values::{FunctionValue, IntValue},
};

use crate::{
    backends::classical::llvm::compiler::{block, codegen::LLVMCodegen, value},
    core::console::logging::{self, LoggingType},
    frontends::classical::{types::ast::Ast, typesystem::types::Type},
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    let abort = |_| {
        self::codegen_abort("Cannot compile while loop at code generation time.");
    };

    if let Ast::While { cond, block, .. } = stmt {
        let condition: BasicBlock = block::append_block(llvm_context, llvm_function);
        let body: BasicBlock = block::append_block(llvm_context, llvm_function);
        let exit: BasicBlock = block::append_block(llvm_context, llvm_function);

        llvm_builder
            .build_unconditional_branch(condition)
            .unwrap_or_else(abort);

        llvm_builder.position_at_end(condition);

        let comparison: IntValue =
            value::compile(codegen.get_mut_context(), cond, Some(&Type::Bool)).into_int_value();

        llvm_builder
            .build_conditional_branch(comparison, body, exit)
            .unwrap_or_else(abort);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_continue_branch(condition);

        codegen
            .get_mut_context()
            .get_mut_loop_ctx()
            .add_break_branch(exit);

        llvm_builder.position_at_end(body);

        codegen.codegen_block(block);

        if codegen
            .get_context()
            .get_last_builder_block()
            .get_terminator()
            .is_none()
        {
            let _ = llvm_builder.build_unconditional_branch(condition);
        }

        llvm_builder.position_at_end(exit);

        codegen.get_mut_context().get_mut_loop_ctx().pop();
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
