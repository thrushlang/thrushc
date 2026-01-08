use thrushc_mir::{atomicord::ThrushAtomicOrdering, threadmode::ThrushThreadMode};

pub trait ModificatorsExtensions {
    fn get_thread_mode(&self) -> Option<ThrushThreadMode>;
    fn get_atomic_ordering(&self) -> Option<ThrushAtomicOrdering>;
    fn has_volatile(&self) -> bool;
    fn has_lazythread(&self) -> bool;
}
