use std::fmt::Display;

use inkwell::InlineAsmDialect;

use crate::{
    backend::{
        llvm::compiler::{attributes::LLVMAttribute, conventions::CallConvention},
        types::traits::AssemblerFunctionExtensions,
    },
    core::console::logging::{self, LoggingType},
};

impl Display for LLVMAttribute<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLVMAttribute::AlwaysInline(..) => write!(f, "@alwaysinline"),
            LLVMAttribute::NoInline(..) => write!(f, "@noinline"),
            LLVMAttribute::InlineHint(..) => write!(f, "@inline"),
            LLVMAttribute::Extern(name, ..) => write!(f, "@extern({})", name),
            LLVMAttribute::Convention(convention, ..) => {
                write!(f, "@convention(\"{}\")", convention)
            }
            LLVMAttribute::Stack(..) => write!(f, "@stack"),
            LLVMAttribute::Heap(..) => write!(f, "@heap"),
            LLVMAttribute::Public(..) => write!(f, "@public"),
            LLVMAttribute::StrongStack(..) => write!(f, "@strongstack"),
            LLVMAttribute::WeakStack(..) => write!(f, "@weakstack"),
            LLVMAttribute::SafeStack(..) => write!(f, "@safestack"),
            LLVMAttribute::PreciseFloats(..) => write!(f, "@precisefp"),
            LLVMAttribute::MinSize(..) => write!(f, "@minsize"),
            LLVMAttribute::Hot(..) => write!(f, "@hot"),
            LLVMAttribute::Ignore(..) => write!(f, "@ignore"),
            LLVMAttribute::AsmThrow(..) => write!(f, "@asmthrow"),
            LLVMAttribute::AsmSyntax(..) => write!(f, "@asmsyntax"),
            LLVMAttribute::AsmSideEffects(..) => write!(f, "@asmeffects"),
            LLVMAttribute::AsmAlignStack(..) => write!(f, "@asmalingstack"),
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
