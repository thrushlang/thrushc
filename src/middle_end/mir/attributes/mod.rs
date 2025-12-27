use crate::core::diagnostic::span::Span;

use crate::front_end::lexer::tokentype::TokenType;
use crate::middle_end::mir::attributes::linkage::ThrushLinkage;

use std::fmt::Display;

pub mod assembler;
pub mod callconventions;
pub mod impls;
pub mod linkage;
pub mod traits;

pub type ThrushAttributes = Vec<ThrushAttribute>;

#[derive(Debug, Clone)]
pub enum ThrushAttribute {
    Extern(String, Span),
    Convention(String, Span),
    Linkage(ThrushLinkage, String, Span),
    Public(Span),
    Ignore(Span),
    Hot(Span),
    NoInline(Span),
    InlineHint(Span),
    MinSize(Span),
    AlwaysInline(Span),
    SafeStack(Span),
    StrongStack(Span),
    WeakStack(Span),
    PreciseFloats(Span),
    NoUnwind(Span),
    OptFuzzing(Span),

    // LLVM Structure Modificator
    Packed(Span),

    // Memory Management
    Stack(Span),
    Heap(Span),

    AsmThrow(Span),
    AsmSyntax(String, Span),
    AsmAlignStack(Span),
    AsmSideEffects(Span),

    //Ctors & Dtors
    Constructor(Span),
    Destructor(Span),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ThrushAttributeComparator {
    Extern,
    Convention,
    Linkage,
    Public,
    Ignore,
    Hot,
    NoInline,
    InlineHint,
    MinSize,
    AlwaysInline,
    SafeStack,
    StrongStack,
    WeakStack,
    PreciseFloats,
    NoUnwind,
    OptFuzzing,

    Packed,

    Stack,
    Heap,

    AsmThrow,
    AsmSyntax,
    AsmAlignStack,
    AsmSideEffects,

    Constructor,
    Destructor,
}

impl ThrushAttribute {
    #[inline]
    pub fn is_extern_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Extern(..))
    }

    #[inline]
    pub fn is_hot_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Hot(..))
    }

    #[inline]
    pub fn is_ignore_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Ignore(..))
    }

    #[inline]
    pub fn is_public_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Public(..))
    }

    #[inline]
    pub fn is_noinline_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::NoInline(..))
    }

    #[inline]
    pub fn is_inline_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::InlineHint(..))
    }

    #[inline]
    pub fn is_alwaysinline_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AlwaysInline(..))
    }

    #[inline]
    pub fn is_minsize_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::MinSize(..))
    }

    #[inline]
    pub fn is_heap_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Heap(..))
    }

    #[inline]
    pub fn is_asmsideeffects_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmSideEffects(..))
    }

    #[inline]
    pub fn is_asmthrow_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmThrow(..))
    }

    #[inline]
    pub fn is_asmalingstack_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmAlignStack(..))
    }

    #[inline]
    pub fn is_asmsyntax_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::AsmSyntax(..))
    }

    #[inline]
    pub fn is_packed(&self) -> bool {
        matches!(self, ThrushAttribute::Packed(..))
    }

    #[inline]
    pub fn is_linkage_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Linkage(..))
    }

    #[inline]
    pub fn is_conv_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Convention(..))
    }

    #[inline]
    pub fn is_constructor_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Constructor(..))
    }

    #[inline]
    pub fn is_destructor_attribute(&self) -> bool {
        matches!(self, ThrushAttribute::Destructor(..))
    }
}

impl ThrushAttribute {
    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            ThrushAttribute::Extern(_, span) => *span,
            ThrushAttribute::Convention(_, span) => *span,
            ThrushAttribute::Linkage(.., span) => *span,
            ThrushAttribute::Public(span) => *span,
            ThrushAttribute::Ignore(span) => *span,
            ThrushAttribute::Hot(span) => *span,
            ThrushAttribute::NoInline(span) => *span,
            ThrushAttribute::InlineHint(span) => *span,
            ThrushAttribute::MinSize(span) => *span,
            ThrushAttribute::AlwaysInline(span) => *span,
            ThrushAttribute::SafeStack(span) => *span,
            ThrushAttribute::StrongStack(span) => *span,
            ThrushAttribute::WeakStack(span) => *span,
            ThrushAttribute::PreciseFloats(span) => *span,
            ThrushAttribute::AsmThrow(span) => *span,
            ThrushAttribute::AsmSyntax(_, span) => *span,
            ThrushAttribute::AsmSideEffects(span) => *span,
            ThrushAttribute::AsmAlignStack(span) => *span,
            ThrushAttribute::Stack(span) => *span,
            ThrushAttribute::Heap(span) => *span,
            ThrushAttribute::Packed(span) => *span,
            ThrushAttribute::NoUnwind(span) => *span,
            ThrushAttribute::OptFuzzing(span) => *span,
            ThrushAttribute::Constructor(span) => *span,
            ThrushAttribute::Destructor(span) => *span,
        }
    }
}

impl ThrushAttribute {
    #[inline]
    pub fn as_llvm_attribute(
        &self,
    ) -> Option<crate::back_end::llvm_codegen::attributes::LLVMAttribute<'_>> {
        match self {
            ThrushAttribute::Extern(external_name, ..) => Some(
                crate::back_end::llvm_codegen::attributes::LLVMAttribute::Extern(external_name),
            ),
            ThrushAttribute::Linkage(linkage, ..) => Some(
                crate::back_end::llvm_codegen::attributes::LLVMAttribute::Linkage(
                    linkage.get_llvm_linkage(),
                ),
            ),
            ThrushAttribute::Convention(name, ..) => Some(
                crate::back_end::llvm_codegen::attributes::LLVMAttribute::Convention(
                    callconventions::get_call_convention(name.as_bytes()),
                ),
            ),
            ThrushAttribute::Public(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Public)
            }
            ThrushAttribute::Ignore(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Ignore)
            }
            ThrushAttribute::Hot(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Hot)
            }
            ThrushAttribute::NoInline(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::NoInline)
            }
            ThrushAttribute::InlineHint(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::InlineHint)
            }
            ThrushAttribute::MinSize(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::MinSize)
            }
            ThrushAttribute::AlwaysInline(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::AlwaysInline)
            }
            ThrushAttribute::SafeStack(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::SafeStack)
            }
            ThrushAttribute::StrongStack(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::StrongStack)
            }
            ThrushAttribute::WeakStack(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::WeakStack)
            }
            ThrushAttribute::PreciseFloats(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::PreciseFloats)
            }
            ThrushAttribute::AsmThrow(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::AsmThrow)
            }
            ThrushAttribute::AsmSyntax(syntax, ..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::AsmSyntax(syntax))
            }
            ThrushAttribute::AsmSideEffects(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::AsmSideEffects)
            }
            ThrushAttribute::AsmAlignStack(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::AsmAlignStack)
            }
            ThrushAttribute::Stack(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Stack)
            }
            ThrushAttribute::Heap(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Heap)
            }
            ThrushAttribute::Packed(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Packed)
            }
            ThrushAttribute::NoUnwind(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::NoUnwind)
            }
            ThrushAttribute::OptFuzzing(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::OptFuzzing)
            }
            ThrushAttribute::Constructor(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Constructor)
            }
            ThrushAttribute::Destructor(..) => {
                Some(crate::back_end::llvm_codegen::attributes::LLVMAttribute::Destructor)
            }
        }
    }
}

#[must_use]
pub fn as_attribute(
    token_type: TokenType,
    span: Span,
) -> Option<crate::middle_end::mir::attributes::ThrushAttribute> {
    match token_type {
        TokenType::Ignore => Some(crate::middle_end::mir::attributes::ThrushAttribute::Ignore(
            span,
        )),
        TokenType::MinSize => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::MinSize(span))
        }
        TokenType::NoInline => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::NoInline(span))
        }
        TokenType::AlwaysInline => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::AlwaysInline(span))
        }
        TokenType::InlineHint => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::InlineHint(span))
        }
        TokenType::Hot => Some(crate::middle_end::mir::attributes::ThrushAttribute::Hot(
            span,
        )),
        TokenType::SafeStack => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::SafeStack(span))
        }
        TokenType::WeakStack => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::WeakStack(span))
        }
        TokenType::StrongStack => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::StrongStack(span))
        }
        TokenType::PreciseFloats => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::PreciseFloats(span))
        }
        TokenType::Stack => Some(crate::middle_end::mir::attributes::ThrushAttribute::Stack(
            span,
        )),
        TokenType::Heap => Some(crate::middle_end::mir::attributes::ThrushAttribute::Heap(
            span,
        )),
        TokenType::AsmThrow => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::AsmThrow(span))
        }
        TokenType::AsmSideEffects => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::AsmSideEffects(span))
        }
        TokenType::AsmAlignStack => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::AsmAlignStack(span))
        }
        TokenType::Packed => Some(crate::middle_end::mir::attributes::ThrushAttribute::Packed(
            span,
        )),
        TokenType::NoUnwind => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::NoUnwind(span))
        }
        TokenType::OptFuzzing => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::OptFuzzing(span))
        }
        TokenType::Constructor => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::Constructor(span))
        }
        TokenType::Destructor => {
            Some(crate::middle_end::mir::attributes::ThrushAttribute::Destructor(span))
        }

        _ => None,
    }
}

impl Display for ThrushAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrushAttribute::AlwaysInline(..) => write!(f, "@alwaysinline"),
            ThrushAttribute::NoInline(..) => write!(f, "@noinline"),
            ThrushAttribute::InlineHint(..) => write!(f, "@inline"),
            ThrushAttribute::Linkage(linkage, ..) => write!(f, "@linkage(\"{}\")", linkage),
            ThrushAttribute::Extern(name, ..) => write!(f, "@extern(\"{}\")", name),
            ThrushAttribute::Convention(convention, ..) => {
                write!(f, "@convention(\"{}\")", convention)
            }
            ThrushAttribute::Stack(..) => write!(f, "@stack"),
            ThrushAttribute::Heap(..) => write!(f, "@heap"),
            ThrushAttribute::Public(..) => write!(f, "@public"),
            ThrushAttribute::StrongStack(..) => write!(f, "@strongstack"),
            ThrushAttribute::WeakStack(..) => write!(f, "@weakstack"),
            ThrushAttribute::SafeStack(..) => write!(f, "@safestack"),
            ThrushAttribute::PreciseFloats(..) => write!(f, "@precisefp"),
            ThrushAttribute::MinSize(..) => write!(f, "@minsize"),
            ThrushAttribute::Hot(..) => write!(f, "@hot"),
            ThrushAttribute::Ignore(..) => write!(f, "@ignore"),
            ThrushAttribute::NoUnwind(..) => write!(f, "@nounwind"),
            ThrushAttribute::AsmThrow(..) => write!(f, "@asmthrow"),
            ThrushAttribute::AsmSyntax(..) => write!(f, "@asmsyntax"),
            ThrushAttribute::AsmSideEffects(..) => write!(f, "@asmeffects"),
            ThrushAttribute::AsmAlignStack(..) => write!(f, "@asmalingstack"),
            ThrushAttribute::Packed(..) => write!(f, "@packed"),
            ThrushAttribute::OptFuzzing(..) => write!(f, "@optfuzzing"),
            ThrushAttribute::Constructor(..) => write!(f, "@constructor"),
            ThrushAttribute::Destructor(..) => write!(f, "@destructor"),
        }
    }
}
