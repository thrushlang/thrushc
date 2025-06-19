use inkwell::context::Context;

use crate::backend::{
    llvm::compiler::context::LLVMCodeGenContext, types::representations::LLVMInstrinsic,
};

#[inline]
pub fn float_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.round.f32",
        llvm_context
            .f32_type()
            .fn_type(&[llvm_context.f32_type().into()], false),
    )
}

#[inline]
pub fn double_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.round.f64",
        llvm_context
            .f64_type()
            .fn_type(&[llvm_context.f64_type().into()], false),
    )
}
