use inkwell::{AddressSpace, context::Context};

use crate::backend::{llvm::compiler::context::LLVMCodeGenContext, types::repr::LLVMInstrinsic};

#[inline]
pub fn start_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.va_start",
        llvm_context.void_type().fn_type(
            &[llvm_context.ptr_type(AddressSpace::default()).into()],
            false,
        ),
    )
}

#[inline]
pub fn copy_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.va_copy",
        llvm_context.void_type().fn_type(
            &[
                llvm_context.ptr_type(AddressSpace::default()).into(),
                llvm_context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        ),
    )
}

#[inline]
pub fn end_instrinsic<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> LLVMInstrinsic<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    (
        "llvm.va_end",
        llvm_context.void_type().fn_type(
            &[llvm_context.ptr_type(AddressSpace::default()).into()],
            false,
        ),
    )
}
