use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::{back_end::llvm::compiler::abort, core::diagnostic::span::Span};

use crate::front_end::typesystem::types::Type;

use inkwell::{context::Context, values::FloatValue};

use std::path::PathBuf;

pub fn generate<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    iee: f64,
    signed: bool,
    span: Span,
) -> FloatValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match kind {
        Type::F32 if signed => llvm_context.f32_type().const_float(-iee),
        Type::F32 => llvm_context.f32_type().const_float(iee),
        Type::F64 if signed => llvm_context.f64_type().const_float(-iee),
        Type::F64 => llvm_context.f64_type().const_float(iee),
        Type::FX8680 if signed => llvm_context.x86_f80_type().const_float(-iee),
        Type::FX8680 => llvm_context.x86_f80_type().const_float(iee),
        Type::F128 if signed => llvm_context.f128_type().const_float(-iee),
        Type::F128 => llvm_context.f128_type().const_float(iee),
        Type::FPPC128 if signed => llvm_context.ppc_f128_type().const_float(-iee),
        Type::FPPC128 => llvm_context.ppc_f128_type().const_float(iee),

        what => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' float type!", what),
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}
