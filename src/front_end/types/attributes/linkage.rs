use inkwell::module::Linkage;

use crate::{core::errors::standard::ThrushCompilerIssue, front_end::lexer::span::Span};

#[derive(Debug, Clone, Copy)]
pub enum ThrushLinkage {
    Standard,
    Common,
    DLLImport,
    DLLExport,
    ExternalWeak,
    Weak,
    Internal,
    LinkerPrivate,
    LinkerPrivateWeak,
}

impl ThrushLinkage {
    #[inline]
    pub fn as_llvm_linkage(&self) -> Linkage {
        match self {
            ThrushLinkage::Standard => Linkage::External,
            ThrushLinkage::Common => Linkage::Common,
            ThrushLinkage::DLLImport => Linkage::DLLImport,
            ThrushLinkage::DLLExport => Linkage::DLLExport,
            ThrushLinkage::ExternalWeak => Linkage::ExternalWeak,
            ThrushLinkage::Weak => Linkage::WeakAny,
            ThrushLinkage::Internal => Linkage::Internal,
            ThrushLinkage::LinkerPrivate => Linkage::LinkerPrivate,
            ThrushLinkage::LinkerPrivateWeak => Linkage::LinkerPrivateWeak,
        }
    }

    #[inline]
    pub fn str_as_thrush_linkage(
        linkage: &str,
        span: Span,
    ) -> Result<ThrushLinkage, ThrushCompilerIssue> {
        match linkage {
            "standard" => Ok(ThrushLinkage::Standard),
            "common" => Ok(ThrushLinkage::Common),
            "dllimport" => Ok(ThrushLinkage::DLLImport),
            "dllexport" => Ok(ThrushLinkage::DLLExport),
            "externweak" => Ok(ThrushLinkage::ExternalWeak),
            "weak" => Ok(ThrushLinkage::Weak),
            "internal" => Ok(ThrushLinkage::Internal),
            "linkerprivate" => Ok(ThrushLinkage::LinkerPrivate),
            "linkerprivateweak" => Ok(ThrushLinkage::LinkerPrivateWeak),

            what => Err(ThrushCompilerIssue::Error(
                "Unknown attribute".into(),
                format!(
                    "The attribute '{}' isn't figured out as a valid linkage mode.",
                    what
                ),
                None,
                span,
            )),
        }
    }
}
