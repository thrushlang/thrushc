use inkwell::AtomicOrdering;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
pub enum ThrushAtomicOrdering {
    AtomicNone,
    AtomicFree,
    AtomicRelax,
    AtomicGrab,
    AtomicDrop,
    AtomicSync,
    AtomicStrict,
}

impl ThrushAtomicOrdering {
    #[inline]
    pub fn to_llvm(self) -> AtomicOrdering {
        match self {
            ThrushAtomicOrdering::AtomicNone => AtomicOrdering::NotAtomic,
            ThrushAtomicOrdering::AtomicFree => AtomicOrdering::Unordered,
            ThrushAtomicOrdering::AtomicRelax => AtomicOrdering::Monotonic,
            ThrushAtomicOrdering::AtomicGrab => AtomicOrdering::Acquire,
            ThrushAtomicOrdering::AtomicDrop => AtomicOrdering::Release,
            ThrushAtomicOrdering::AtomicSync => AtomicOrdering::AcquireRelease,
            ThrushAtomicOrdering::AtomicStrict => AtomicOrdering::SequentiallyConsistent,
        }
    }
}
