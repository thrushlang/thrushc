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
pub enum ThrustLinkage {
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

impl ThrustLinkage {
    #[inline]
    pub fn is_standard(&self) -> bool {
        matches!(self, ThrustLinkage::Standard)
    }

    #[inline]
    pub fn is_linker_private(&self) -> bool {
        matches!(self, ThrustLinkage::LinkerPrivate)
    }

    #[inline]
    pub fn is_linker_private_weak(&self) -> bool {
        matches!(self, ThrustLinkage::LinkerPrivateWeak)
    }

    #[inline]
    pub fn is_internal(&self) -> bool {
        matches!(self, ThrustLinkage::Internal)
    }
}

impl ThrustLinkage {
    #[inline]
    pub fn get_llvm_linkage(&self) -> Linkage {
        match self {
            ThrustLinkage::Standard => Linkage::External,
            ThrustLinkage::Common => Linkage::Common,
            ThrustLinkage::DLLImport => Linkage::DLLImport,
            ThrustLinkage::DLLExport => Linkage::DLLExport,
            ThrustLinkage::ExternalWeak => Linkage::ExternalWeak,
            ThrustLinkage::Weak => Linkage::WeakAny,
            ThrustLinkage::Internal => Linkage::Internal,
            ThrustLinkage::LinkerPrivate => Linkage::LinkerPrivate,
            ThrustLinkage::LinkerPrivateWeak => Linkage::LinkerPrivateWeak,
        }
    }

    #[inline]
    pub fn get_linkage(id: &str) -> ThrustLinkage {
        match id {
            "standard" => ThrustLinkage::Standard,
            "common" => ThrustLinkage::Common,
            "dllimport" => ThrustLinkage::DLLImport,
            "dllexport" => ThrustLinkage::DLLExport,
            "externweak" => ThrustLinkage::ExternalWeak,
            "weak" => ThrustLinkage::Weak,
            "internal" => ThrustLinkage::Internal,
            "linkerprivate" => ThrustLinkage::LinkerPrivate,
            "linkerprivateweak" => ThrustLinkage::LinkerPrivateWeak,

            _ => ThrustLinkage::Standard,
        }
    }
}

impl Display for ThrustLinkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrustLinkage::Standard => write!(f, "standard"),
            ThrustLinkage::Common => write!(f, "common"),
            ThrustLinkage::DLLImport => write!(f, "dllimport"),
            ThrustLinkage::DLLExport => write!(f, "dllexport"),
            ThrustLinkage::ExternalWeak => write!(f, "externweak"),
            ThrustLinkage::Weak => write!(f, "weak"),
            ThrustLinkage::Internal => write!(f, "internal"),
            ThrustLinkage::LinkerPrivate => write!(f, "linkerprivate"),
            ThrustLinkage::LinkerPrivateWeak => write!(f, "linkerprivateweak"),
        }
    }
}
