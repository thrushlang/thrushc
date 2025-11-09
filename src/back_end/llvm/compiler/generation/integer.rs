use std::path::PathBuf;

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::{back_end::llvm::compiler::abort, front_end::lexer::span::Span};

use crate::front_end::typesystem::types::Type;

use inkwell::{context::Context, values::IntValue};

pub fn generate<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    number: u64,
    signed: bool,
    span: Span,
) -> IntValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match kind {
        Type::Char => llvm_context.i8_type().const_int(number, signed).const_neg(),
        Type::S8 if signed => llvm_context.i8_type().const_int(number, signed).const_neg(),
        Type::S8 => llvm_context.i8_type().const_int(number, signed),
        Type::S16 if signed => llvm_context
            .i16_type()
            .const_int(number, signed)
            .const_neg(),
        Type::S16 => llvm_context.i16_type().const_int(number, signed),
        Type::S32 if signed => llvm_context
            .i32_type()
            .const_int(number, signed)
            .const_neg(),
        Type::S32 => llvm_context.i32_type().const_int(number, signed),
        Type::S64 if signed => llvm_context
            .i64_type()
            .const_int(number, signed)
            .const_neg(),
        Type::S64 => llvm_context.i64_type().const_int(number, signed),
        Type::U8 => llvm_context.i8_type().const_int(number, false),
        Type::U16 => llvm_context.i16_type().const_int(number, false),
        Type::U32 => llvm_context.i32_type().const_int(number, false),
        Type::U64 => llvm_context.i64_type().const_int(number, false),
        Type::U128 if signed => llvm_context
            .i128_type()
            .const_int(number, signed)
            .const_neg(),
        Type::U128 => llvm_context.i128_type().const_int(number, signed),
        Type::Bool => llvm_context.bool_type().const_int(number, false),

        what => abort::abort_codegen(
            context,
            &format!("Failed to compile '{}' integer type!", what),
            span,
            PathBuf::from(file!()),
            line!(),
        ),
    }
}
