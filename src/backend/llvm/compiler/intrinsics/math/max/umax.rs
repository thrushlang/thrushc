use inkwell::context::Context;

use crate::backend::{
    llvm::compiler::context::LLVMCodeGenContext, types::representations::LLVMInstrinsic,
};

#[inline]
pub fn i8_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.umax.i8",
        llvm_context.i8_type().fn_type(
            &[llvm_context.i8_type().into(), llvm_context.i8_type().into()],
            false,
        ),
    )
}

#[inline]
pub fn i16_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.umax.i16",
        llvm_context.i16_type().fn_type(
            &[
                llvm_context.i16_type().into(),
                llvm_context.i16_type().into(),
            ],
            false,
        ),
    )
}

#[inline]
pub fn i32_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.umax.i32",
        llvm_context.i32_type().fn_type(
            &[
                llvm_context.i32_type().into(),
                llvm_context.i32_type().into(),
            ],
            false,
        ),
    )
}

#[inline]
pub fn i64_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.umax.i64",
        llvm_context.i64_type().fn_type(
            &[
                llvm_context.i64_type().into(),
                llvm_context.i64_type().into(),
            ],
            false,
        ),
    )
}
