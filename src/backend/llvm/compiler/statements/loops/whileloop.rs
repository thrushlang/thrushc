use std::fmt::Display;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    values::{FunctionValue, IntValue},
};

use crate::{
    backend::llvm::compiler::{codegen::LLVMCodegen, valuegen},
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, stmt: &'ctx Ast<'ctx>) {
    let llvm_context: &Context = codegen.get_mut_context().get_llvm_context();
    let llvm_builder: &Builder = codegen.get_mut_context().get_llvm_builder();

    let llvm_function: FunctionValue = codegen.get_mut_context().get_current_fn();

    let abort = |_| {
        self::codegen_abort("Cannot compile while loop at code generation time.");
        unreachable!()
    };

    if let Ast::While { cond, block, .. } = stmt {
        let condition: BasicBlock = llvm_context.append_basic_block(llvm_function, "while");
        let body: BasicBlock = llvm_context.append_basic_block(llvm_function, "while_body");
        let exit: BasicBlock = llvm_context.append_basic_block(llvm_function, "while_exit");

        llvm_builder
            .build_unconditional_branch(condition)
            .unwrap_or_else(abort);

        llvm_builder.position_at_end(condition);

        let comparison: IntValue =
            valuegen::compile(codegen.get_mut_context(), cond, Some(&Type::Bool)).into_int_value();

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
    } else {
        self::codegen_abort("Expected while loop to compile.");
    }
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
