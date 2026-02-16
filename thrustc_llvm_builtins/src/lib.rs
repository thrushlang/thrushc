use thrustc_ast::{Ast, builitins::ThrustBuiltin};
use thrustc_span::Span;
use thrustc_typesystem::Type;

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

pub fn into_llvm_builtin<'ctx>(thrust_builtin: &'ctx ThrustBuiltin) -> LLVMBuiltin<'ctx> {
    match thrust_builtin {
        ThrustBuiltin::Halloc { of, span } => LLVMBuiltin::Malloc { of, span: *span },
        ThrustBuiltin::MemCpy {
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
        ThrustBuiltin::MemMove {
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
        ThrustBuiltin::MemSet {
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
        ThrustBuiltin::AlignOf { of, span } => LLVMBuiltin::AlignOf { of, span: *span },
        ThrustBuiltin::SizeOf { of, span } => LLVMBuiltin::SizeOf { of, span: *span },
        ThrustBuiltin::BitSizeOf { of, span } => LLVMBuiltin::BitSizeOf { of, span: *span },
        ThrustBuiltin::AbiSizeOf { of, span } => LLVMBuiltin::AbiSizeOf { of, span: *span },
        ThrustBuiltin::AbiAlignOf { of, span } => LLVMBuiltin::AbiAlignOf { of, span: *span },
    }
}
