mod impls;
pub mod traits;

use thrushc_mir::{atomicord::ThrushAtomicOrdering, threadmode::ThrushThreadMode};

pub type Modificators = Vec<Modificator>;

#[derive(Debug, Clone, Copy)]
pub enum Modificator {
    ThreadMode(ThrushThreadMode),
    AtomicOrdering(ThrushAtomicOrdering),
    Volatile,
    LazyThread,

    None,
}

impl Modificator {
    pub fn is_thread_mode(&self) -> bool {
        matches!(self, Modificator::ThreadMode(..))
    }

    pub fn is_atomic_ordering(&self) -> bool {
        matches!(self, Modificator::AtomicOrdering(..))
    }

    pub fn is_volatile(&self) -> bool {
        matches!(self, Modificator::Volatile)
    }

    pub fn is_lazy_thread(&self) -> bool {
        matches!(self, Modificator::LazyThread)
    }
}
