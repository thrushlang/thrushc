use inkwell::module::Linkage;
use std::fmt::Display;

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

pub const LINKAGES_AVAILABLE: &[&str] = &[
    "standard",
    "common",
    "dllimport",
    "dllexport",
    "externweak",
    "weak",
    "internal",
    "linkerprivate",
    "linkerprivateweak",
];

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
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
    pub fn is_standard(&self) -> bool {
        matches!(self, ThrushLinkage::Standard)
    }

    #[inline]
    pub fn is_linker_private(&self) -> bool {
        matches!(self, ThrushLinkage::LinkerPrivate)
    }

    #[inline]
    pub fn is_linker_private_weak(&self) -> bool {
        matches!(self, ThrushLinkage::LinkerPrivateWeak)
    }

    #[inline]
    pub fn is_internal(&self) -> bool {
        matches!(self, ThrushLinkage::Internal)
    }
}

impl ThrushLinkage {
    #[inline]
    pub fn get_llvm_linkage(&self) -> Linkage {
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
    pub fn get_linkage(id: &str) -> ThrushLinkage {
        match id {
            "standard" => ThrushLinkage::Standard,
            "common" => ThrushLinkage::Common,
            "dllimport" => ThrushLinkage::DLLImport,
            "dllexport" => ThrushLinkage::DLLExport,
            "externweak" => ThrushLinkage::ExternalWeak,
            "weak" => ThrushLinkage::Weak,
            "internal" => ThrushLinkage::Internal,
            "linkerprivate" => ThrushLinkage::LinkerPrivate,
            "linkerprivateweak" => ThrushLinkage::LinkerPrivateWeak,

            _ => ThrushLinkage::Standard,
        }
    }
}

impl Display for ThrushLinkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrushLinkage::Standard => write!(f, "standard"),
            ThrushLinkage::Common => write!(f, "common"),
            ThrushLinkage::DLLImport => write!(f, "dllimport"),
            ThrushLinkage::DLLExport => write!(f, "dllexport"),
            ThrushLinkage::ExternalWeak => write!(f, "externweak"),
            ThrushLinkage::Weak => write!(f, "weak"),
            ThrushLinkage::Internal => write!(f, "internal"),
            ThrushLinkage::LinkerPrivate => write!(f, "linkerprivate"),
            ThrushLinkage::LinkerPrivateWeak => write!(f, "linkerprivateweak"),
        }
    }
}
