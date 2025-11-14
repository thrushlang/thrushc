use crate::front_end::types::attributes::ThrushAttribute;

use super::traits::ThrushAttributeComparatorExtensions;
use super::types::ThrushAttributeComparator;

impl ThrushAttributeComparatorExtensions for ThrushAttribute {
    #[inline]
    fn into_attr_cmp(&self) -> ThrushAttributeComparator {
        match self {
            ThrushAttribute::Extern(..) => ThrushAttributeComparator::Extern,
            ThrushAttribute::Convention(..) => ThrushAttributeComparator::Convention,
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
        }
    }
}
