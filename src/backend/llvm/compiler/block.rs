use inkwell::{basic_block::BasicBlock, values::FunctionValue};

use crate::backend::llvm::compiler::context::LLVMCodeGenContext;

pub fn move_terminator_to_end(context: &LLVMCodeGenContext) {
    let function: FunctionValue = context.get_current_fn();

    let last_builder_block: BasicBlock = context.get_last_builder_block();

    if let Some(parent) = function.get_last_basic_block() {
        let _ = last_builder_block.move_after(parent);
    }
}
