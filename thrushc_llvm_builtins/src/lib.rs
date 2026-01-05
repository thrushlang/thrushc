use thrushc_ast::{Ast, builitins::ThrushBuiltin};
use thrushc_span::Span;
use thrushc_typesystem::Type;

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
    AbiSizeOf {
        of: &'ctx Type,
        span: Span,
    },
    BitSizeOf {
        of: &'ctx Type,
        span: Span,
    },
    AbiAlignOf {
        of: &'ctx Type,
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

pub fn into_llvm_builtin<'ctx>(thrush_builtin: &'ctx ThrushBuiltin) -> LLVMBuiltin<'ctx> {
    match thrush_builtin {
        ThrushBuiltin::Halloc { of, span } => LLVMBuiltin::Malloc { of, span: *span },
        ThrushBuiltin::MemCpy {
            src,
            dst,
            size,
            span,
        } => LLVMBuiltin::MemCpy {
            src: src.as_ref(),
            dst,
            size,
            span: *span,
        },
        ThrushBuiltin::MemMove {
            src,
            dst,
            size,
            span,
        } => LLVMBuiltin::MemMove {
            src,
            dst,
            size,
            span: *span,
        },
        ThrushBuiltin::MemSet {
            dst,
            new_size,
            size,
            span,
        } => LLVMBuiltin::MemSet {
            dst,
            new_size,
            size,
            span: *span,
        },
        ThrushBuiltin::AlignOf { of, span } => LLVMBuiltin::AlignOf { of, span: *span },
        ThrushBuiltin::SizeOf { of, span } => LLVMBuiltin::SizeOf { of, span: *span },
        ThrushBuiltin::BitSizeOf { of, span } => LLVMBuiltin::BitSizeOf { of, span: *span },
        ThrushBuiltin::AbiSizeOf { of, span } => LLVMBuiltin::AbiSizeOf { of, span: *span },
        ThrushBuiltin::AbiAlignOf { of, span } => LLVMBuiltin::AbiAlignOf { of, span: *span },
    }
}
