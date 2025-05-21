use inkwell::values::BasicValueEnum;

use crate::backend::llvm::compiler::context::LLVMCodeGenContext;

pub trait LLVMDeallocator {
    fn dealloc(&self, context: &LLVMCodeGenContext<'_, '_>, value: BasicValueEnum<'_>);
}
