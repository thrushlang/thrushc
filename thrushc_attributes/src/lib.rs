use thrushc_span::Span;
use thrushc_token_type::TokenType;

use crate::{
    linkage::ThrushLinkage,
    traits::{ThrushAttributeComparatorExtensions, ThrushAttributesExtensions},
};

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

pub mod assembler;
pub mod callconventions;
pub mod linkage;
pub mod traits;

pub type ThrushAttributes = Vec<ThrushAttribute>;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
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
    Pure(Span),
    Thunk(Span),

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
    Pure,
    Thunk,

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
            ThrushAttribute::Pure(span) => *span,
            ThrushAttribute::Thunk(span) => *span,
            ThrushAttribute::Constructor(span) => *span,
            ThrushAttribute::Destructor(span) => *span,
        }
    }
}

#[must_use]
pub fn as_attribute(token_type: TokenType, span: Span) -> Option<ThrushAttribute> {
    match token_type {
        TokenType::Ignore => Some(ThrushAttribute::Ignore(span)),
        TokenType::MinSize => Some(ThrushAttribute::MinSize(span)),
        TokenType::NoInline => Some(ThrushAttribute::NoInline(span)),
        TokenType::AlwaysInline => Some(ThrushAttribute::AlwaysInline(span)),
        TokenType::InlineHint => Some(ThrushAttribute::InlineHint(span)),
        TokenType::Hot => Some(ThrushAttribute::Hot(span)),
        TokenType::SafeStack => Some(ThrushAttribute::SafeStack(span)),
        TokenType::WeakStack => Some(ThrushAttribute::WeakStack(span)),
        TokenType::StrongStack => Some(ThrushAttribute::StrongStack(span)),
        TokenType::PreciseFloats => Some(ThrushAttribute::PreciseFloats(span)),
        TokenType::Heap => Some(ThrushAttribute::Heap(span)),
        TokenType::AsmThrow => Some(ThrushAttribute::AsmThrow(span)),
        TokenType::AsmSideEffects => Some(ThrushAttribute::AsmSideEffects(span)),
        TokenType::AsmAlignStack => Some(ThrushAttribute::AsmAlignStack(span)),
        TokenType::Packed => Some(ThrushAttribute::Packed(span)),
        TokenType::NoUnwind => Some(ThrushAttribute::NoUnwind(span)),
        TokenType::OptFuzzing => Some(ThrushAttribute::OptFuzzing(span)),
        TokenType::Pure => Some(ThrushAttribute::Pure(span)),
        TokenType::Thunk => Some(ThrushAttribute::Thunk(span)),
        TokenType::Constructor => Some(ThrushAttribute::Constructor(span)),
        TokenType::Destructor => Some(ThrushAttribute::Destructor(span)),

        _ => None,
    }
}

impl ThrushAttributesExtensions for ThrushAttributes {
    #[inline]
    fn has_linkage_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_linkage_attribute())
    }

    #[inline]
    fn has_extern_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_extern_attribute())
    }

    #[inline]
    fn has_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    #[inline]
    fn has_heap_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_heap_attribute())
    }

    #[inline]
    fn has_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }

    #[inline]
    fn has_hot_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_hot_attribute())
    }

    #[inline]
    fn has_inline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_inline_attribute())
    }

    #[inline]
    fn has_minsize_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_minsize_attribute())
    }

    #[inline]
    fn has_inlinealways_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_alwaysinline_attribute())
    }

    #[inline]
    fn has_noinline_attr(&self) -> bool {
        self.iter().any(|attr| attr.is_noinline_attribute())
    }

    #[inline]
    fn has_asmalignstack_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmalingstack_attribute())
    }

    #[inline]
    fn has_asmsideffects_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsideeffects_attribute())
    }

    #[inline]
    fn has_asmthrow_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmthrow_attribute())
    }

    #[inline]
    fn has_asmsyntax_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_asmsyntax_attribute())
    }

    #[inline]
    fn has_convention_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_conv_attribute())
    }

    #[inline]
    fn has_constructor_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_constructor_attribute())
    }

    #[inline]
    fn has_destructor_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_destructor_attribute())
    }

    #[inline]
    fn match_attr(&self, cmp: ThrushAttributeComparator) -> Option<Span> {
        if let Some(attr_found) = self.iter().find(|attr| attr.as_attr_cmp() == cmp) {
            return Some(attr_found.get_span());
        }

        None
    }

    #[inline]
    fn get_attr(&self, cmp: ThrushAttributeComparator) -> Option<ThrushAttribute> {
        if let Some(attr_found) = self.iter().find(|attr| attr.as_attr_cmp() == cmp) {
            return Some(attr_found.clone());
        }

        None
    }
}

impl ThrushAttributeComparatorExtensions for ThrushAttribute {
    #[inline]
    fn as_attr_cmp(&self) -> ThrushAttributeComparator {
        match self {
            ThrushAttribute::Extern(..) => ThrushAttributeComparator::Extern,
            ThrushAttribute::Convention(..) => ThrushAttributeComparator::Convention,
            ThrushAttribute::Linkage(..) => ThrushAttributeComparator::Linkage,
            ThrushAttribute::Stack(..) => ThrushAttributeComparator::Stack,
            ThrushAttribute::Heap(..) => ThrushAttributeComparator::Heap,
            ThrushAttribute::Public(..) => ThrushAttributeComparator::Public,
            ThrushAttribute::Ignore(..) => ThrushAttributeComparator::Ignore,
            ThrushAttribute::Hot(..) => ThrushAttributeComparator::Hot,
            ThrushAttribute::NoInline(..) => ThrushAttributeComparator::NoInline,
            ThrushAttribute::InlineHint(..) => ThrushAttributeComparator::InlineHint,
            ThrushAttribute::MinSize(..) => ThrushAttributeComparator::MinSize,
            ThrushAttribute::AlwaysInline(..) => ThrushAttributeComparator::AlwaysInline,
            ThrushAttribute::SafeStack(_) => ThrushAttributeComparator::SafeStack,
            ThrushAttribute::StrongStack(..) => ThrushAttributeComparator::StrongStack,
            ThrushAttribute::WeakStack(..) => ThrushAttributeComparator::WeakStack,
            ThrushAttribute::PreciseFloats(..) => ThrushAttributeComparator::PreciseFloats,
            ThrushAttribute::AsmAlignStack(..) => ThrushAttributeComparator::AsmAlignStack,
            ThrushAttribute::AsmSyntax(..) => ThrushAttributeComparator::AsmSyntax,
            ThrushAttribute::AsmThrow(..) => ThrushAttributeComparator::AsmThrow,
            ThrushAttribute::AsmSideEffects(..) => ThrushAttributeComparator::AsmSideEffects,
            ThrushAttribute::Packed(..) => ThrushAttributeComparator::Packed,
            ThrushAttribute::NoUnwind(..) => ThrushAttributeComparator::NoUnwind,
            ThrushAttribute::OptFuzzing(..) => ThrushAttributeComparator::OptFuzzing,
            ThrushAttribute::Pure(..) => ThrushAttributeComparator::Pure,
            ThrushAttribute::Thunk(..) => ThrushAttributeComparator::Thunk,
            ThrushAttribute::Constructor(..) => ThrushAttributeComparator::Constructor,
            ThrushAttribute::Destructor(..) => ThrushAttributeComparator::Destructor,
        }
    }
}

impl std::fmt::Display for ThrushAttribute {
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
            ThrushAttribute::Pure(..) => write!(f, "@pure"),
            ThrushAttribute::Thunk(..) => write!(f, "@thunk"),
            ThrushAttribute::Constructor(..) => write!(f, "@constructor"),
            ThrushAttribute::Destructor(..) => write!(f, "@destructor"),
        }
    }
}
