use thrustc_mir::{atomicord::ThrustAtomicOrdering, threadmode::ThrustThreadMode};

pub trait ModificatorsExtensions {
    fn get_thread_mode(&self) -> Option<ThrustThreadMode>;
    fn get_atomic_ordering(&self) -> Option<ThrustAtomicOrdering>;
    fn has_volatile(&self) -> bool;
    fn has_lazythread(&self) -> bool;
}
