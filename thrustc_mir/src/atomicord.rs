use inkwell::AtomicOrdering;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
pub enum ThrustAtomicOrdering {
    AtomicNone,
    AtomicFree,
    AtomicRelax,
    AtomicGrab,
    AtomicDrop,
    AtomicSync,
    AtomicStrict,
}

impl ThrustAtomicOrdering {
    #[inline]
    pub fn to_llvm(self) -> AtomicOrdering {
        match self {
            ThrustAtomicOrdering::AtomicNone => AtomicOrdering::NotAtomic,
            ThrustAtomicOrdering::AtomicFree => AtomicOrdering::Unordered,
            ThrustAtomicOrdering::AtomicRelax => AtomicOrdering::Monotonic,
            ThrustAtomicOrdering::AtomicGrab => AtomicOrdering::Acquire,
            ThrustAtomicOrdering::AtomicDrop => AtomicOrdering::Release,
            ThrustAtomicOrdering::AtomicSync => AtomicOrdering::AcquireRelease,
            ThrustAtomicOrdering::AtomicStrict => AtomicOrdering::SequentiallyConsistent,
        }
    }
}
