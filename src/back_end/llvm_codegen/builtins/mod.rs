use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;

pub mod mem;

#[derive(Debug, Clone)]
pub enum LLVMBuiltin<'ctx> {
    Malloc {
        of: &'ctx Type,
        span: Span,
    },
    MemCpy {
        src: &'ctx Ast<'ctx>,
        dst: &'ctx Ast<'ctx>,
        size: &'ctx Ast<'ctx>,
        span: Span,
    },
    MemMove {
        src: &'ctx Ast<'ctx>,
        dst: &'ctx Ast<'ctx>,
        size: &'ctx Ast<'ctx>,
        span: Span,
    },
    MemSet {
        dst: &'ctx Ast<'ctx>,
        new_size: &'ctx Ast<'ctx>,
        size: &'ctx Ast<'ctx>,
        span: Span,
    },
    AlignOf {
        of: &'ctx Type,
        span: Span,
    },
    SizeOf {
        of: &'ctx Type,
        span: Span,
    },
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    builtin: LLVMBuiltin<'ctx>,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match builtin {
        LLVMBuiltin::AlignOf { of, span } => mem::alingof::compile(context, of, span, cast_type),
        LLVMBuiltin::MemCpy {
            src,
            dst,
            size,
            span,
        } => mem::memcpy::compile(context, src, dst, size, span),
        LLVMBuiltin::MemMove {
            src,
            dst,
            size,
            span,
        } => mem::memmove::compile(context, src, dst, size, span),
        LLVMBuiltin::MemSet {
            dst,
            new_size,
            size,
            span,
        } => mem::memset::compile(context, dst, new_size, size, span),
        LLVMBuiltin::Malloc { of, span } => mem::malloc::compile(context, of, span),
        LLVMBuiltin::SizeOf { of, span } => mem::sizeof::compile(context, of, cast_type, span),
    }
}
