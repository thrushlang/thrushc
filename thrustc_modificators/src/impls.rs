use thrustc_mir::{atomicord::ThrustAtomicOrdering, threadmode::ThrustThreadMode};

use crate::{Modificator, Modificators, traits::ModificatorsExtensions};

impl ModificatorsExtensions for Modificators {
    fn get_atomic_ordering(&self) -> Option<ThrustAtomicOrdering> {
        self.iter().find_map(|m| match m {
            Modificator::AtomicOrdering(atomic_ord) => Some(*atomic_ord),
            _ => None,
        })
    }

    fn get_thread_mode(&self) -> Option<ThrustThreadMode> {
        self.iter().find_map(|m| match m {
            Modificator::ThreadMode(thread_mode) => Some(*thread_mode),
            _ => None,
        })
    }

    fn has_lazythread(&self) -> bool {
        self.iter().any(|m| m.is_lazy_thread())
    }

    fn has_volatile(&self) -> bool {
        self.iter().any(|m| m.is_volatile())
    }
}
