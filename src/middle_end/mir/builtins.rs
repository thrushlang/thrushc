use crate::{
    core::diagnostic::span::Span,
    front_end::{types::ast::Ast, typesystem::types::Type},
};

#[derive(Debug, Clone)]
pub enum ThrushBuiltin<'mir> {
    Halloc {
        of: Type,
        span: Span,
    },
    MemCpy {
        src: std::boxed::Box<Ast<'mir>>,
        dst: std::boxed::Box<Ast<'mir>>,
        size: std::boxed::Box<Ast<'mir>>,
        span: Span,
    },
    MemMove {
        src: std::boxed::Box<Ast<'mir>>,
        dst: std::boxed::Box<Ast<'mir>>,
        size: std::boxed::Box<Ast<'mir>>,
        span: Span,
    },
    MemSet {
        dst: std::boxed::Box<Ast<'mir>>,
        new_size: std::boxed::Box<Ast<'mir>>,
        size: std::boxed::Box<Ast<'mir>>,
        span: Span,
    },
    BitSizeOf {
        of: Type,
        span: Span,
    },
    AbiSizeOf {
        of: Type,
        span: Span,
    },
    AlignOf {
        of: Type,
        span: Span,
    },
    SizeOf {
        of: Type,
        span: Span,
    },
}

impl<'mir> ThrushBuiltin<'mir> {
    pub fn to_llvm_builtin(&self) -> crate::back_end::llvm_codegen::builtins::LLVMBuiltin {
        match self {
            ThrushBuiltin::Halloc { of, span } => {
                crate::back_end::llvm_codegen::builtins::LLVMBuiltin::Malloc { of, span: *span }
            }
            ThrushBuiltin::MemCpy {
                src,
                dst,
                size,
                span,
            } => crate::back_end::llvm_codegen::builtins::LLVMBuiltin::MemCpy {
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
            } => crate::back_end::llvm_codegen::builtins::LLVMBuiltin::MemMove {
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
            } => crate::back_end::llvm_codegen::builtins::LLVMBuiltin::MemSet {
                dst,
                new_size,
                size,
                span: *span,
            },

            ThrushBuiltin::AlignOf { of, span } => {
                crate::back_end::llvm_codegen::builtins::LLVMBuiltin::AlignOf { of, span: *span }
            }
            ThrushBuiltin::SizeOf { of, span } => {
                crate::back_end::llvm_codegen::builtins::LLVMBuiltin::SizeOf { of, span: *span }
            }
            ThrushBuiltin::BitSizeOf { of, span } => {
                crate::back_end::llvm_codegen::builtins::LLVMBuiltin::BitSizeOf { of, span: *span }
            }
            ThrushBuiltin::AbiSizeOf { of, span } => {
                crate::back_end::llvm_codegen::builtins::LLVMBuiltin::AbiSizeOf { of, span: *span }
            }
        }
    }
}
