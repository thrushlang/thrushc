use std::fmt::Display;

use inkwell::InlineAsmDialect;

use crate::{
    back_end::llvm::{
        compiler::{
            attributes::{LLVMAttribute, LLVMAttributeComparator},
            conventions::CallConvention,
        },
        types::{
            repr::LLVMAttributes,
            traits::{
                AssemblerFunctionExtensions, LLVMAttributeComparatorExtensions,
                LLVMAttributesExtensions,
            },
        },
    },
    core::console::logging::{self, LoggingType},
};

impl LLVMAttributesExtensions for LLVMAttributes<'_> {
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
    fn get_attr(&self, cmp: LLVMAttributeComparator) -> Option<LLVMAttribute<'_>> {
        if let Some(attr_found) = self.iter().find(|attr| attr.as_attr_cmp() == cmp) {
            return Some(*attr_found);
        }

        None
    }
}

impl LLVMAttributeComparatorExtensions for LLVMAttribute<'_> {
    fn as_attr_cmp(&self) -> LLVMAttributeComparator {
        match self {
            LLVMAttribute::Extern(..) => LLVMAttributeComparator::Extern,
            LLVMAttribute::Convention(..) => LLVMAttributeComparator::Convention,
            LLVMAttribute::Stack => LLVMAttributeComparator::Stack,
            LLVMAttribute::Heap => LLVMAttributeComparator::Heap,
            LLVMAttribute::Public => LLVMAttributeComparator::Public,
            LLVMAttribute::Ignore => LLVMAttributeComparator::Ignore,
            LLVMAttribute::Hot => LLVMAttributeComparator::Hot,
            LLVMAttribute::NoInline => LLVMAttributeComparator::NoInline,
            LLVMAttribute::InlineHint => LLVMAttributeComparator::InlineHint,
            LLVMAttribute::MinSize => LLVMAttributeComparator::MinSize,
            LLVMAttribute::AlwaysInline => LLVMAttributeComparator::AlwaysInline,
            LLVMAttribute::SafeStack => LLVMAttributeComparator::SafeStack,
            LLVMAttribute::StrongStack => LLVMAttributeComparator::StrongStack,
            LLVMAttribute::WeakStack => LLVMAttributeComparator::WeakStack,
            LLVMAttribute::PreciseFloats => LLVMAttributeComparator::PreciseFloats,
            LLVMAttribute::AsmAlignStack => LLVMAttributeComparator::AsmAlignStack,
            LLVMAttribute::AsmSyntax(..) => LLVMAttributeComparator::AsmSyntax,
            LLVMAttribute::AsmThrow => LLVMAttributeComparator::AsmThrow,
            LLVMAttribute::AsmSideEffects => LLVMAttributeComparator::AsmSideEffects,
            LLVMAttribute::Packed => LLVMAttributeComparator::Packed,
            LLVMAttribute::NoUnwind => LLVMAttributeComparator::NoUnwind,
            LLVMAttribute::OptFuzzing => LLVMAttributeComparator::OptFuzzing,
        }
    }
}

impl Display for LLVMAttribute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLVMAttribute::AlwaysInline => write!(f, "@alwaysinline"),
            LLVMAttribute::NoInline => write!(f, "@noinline"),
            LLVMAttribute::InlineHint => write!(f, "@inline"),
            LLVMAttribute::Extern(name, ..) => write!(f, "@extern({})", name),
            LLVMAttribute::Convention(convention, ..) => {
                write!(f, "@convention(\"{}\")", convention)
            }
            LLVMAttribute::Stack => write!(f, "@stack"),
            LLVMAttribute::Heap => write!(f, "@heap"),
            LLVMAttribute::Public => write!(f, "@public"),
            LLVMAttribute::StrongStack => write!(f, "@strongstack"),
            LLVMAttribute::WeakStack => write!(f, "@weakstack"),
            LLVMAttribute::SafeStack => write!(f, "@safestack"),
            LLVMAttribute::PreciseFloats => write!(f, "@precisefp"),
            LLVMAttribute::MinSize => write!(f, "@minsize"),
            LLVMAttribute::Hot => write!(f, "@hot"),
            LLVMAttribute::Ignore => write!(f, "@ignore"),
            LLVMAttribute::NoUnwind => write!(f, "@nounwind"),
            LLVMAttribute::AsmThrow => write!(f, "@asmthrow"),
            LLVMAttribute::AsmSyntax(..) => write!(f, "@asmsyntax"),
            LLVMAttribute::AsmSideEffects => write!(f, "@asmeffects"),
            LLVMAttribute::AsmAlignStack => write!(f, "@asmalingstack"),
            LLVMAttribute::Packed => write!(f, "@packed"),
            LLVMAttribute::OptFuzzing => write!(f, "@optfuzzing"),
        }
    }
}

impl Display for CallConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallConvention::Standard => write!(f, "C"),
            CallConvention::Fast => write!(f, "fast"),
            CallConvention::Cold => write!(f, "cold"),
            CallConvention::GHC => write!(f, "haskell"),
            CallConvention::PreserveAll => write!(f, "strongReg"),
            CallConvention::PreserveMost => write!(f, "weakReg"),
            CallConvention::Tail => write!(f, "tail"),
            CallConvention::Swift => write!(f, "swift"),
            CallConvention::HiPE => write!(f, "erlang"),
        }
    }
}

impl AssemblerFunctionExtensions for str {
    #[inline]
    fn to_inline_assembler_dialect(syntax: &str) -> InlineAsmDialect {
        match syntax {
            "Intel" => InlineAsmDialect::Intel,
            "AT&T" => InlineAsmDialect::ATT,
            any => {
                logging::print_backend_bug(
                    LoggingType::BackendBug,
                    &format!(
                        "Unable to translate '{}' to proper inline assembler dialect.",
                        any
                    ),
                );
            }
        }
    }
}
