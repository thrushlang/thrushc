use ahash::{HashMap, HashSet};

use super::memory::AllocatedSymbol;

pub type AllocatedSymbols<'ctx> = &'ctx HashMap<&'ctx str, AllocatedSymbol<'ctx>>;
pub type MappedHeapPointers<'ctx> = HashSet<(&'ctx str, u32, bool)>;
pub type MappedHeapPointer<'ctx> = (&'ctx str, u32, bool);
