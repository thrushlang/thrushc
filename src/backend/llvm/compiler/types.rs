use ahash::{AHashMap as HashMap, HashSet};

use super::memory::SymbolAllocated;

pub type SymbolsAllocated<'ctx> = &'ctx HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type MappedHeapPointers<'ctx> = HashSet<(&'ctx str, u32, bool)>;
pub type MappedHeapPointer<'ctx> = (&'ctx str, u32, bool);
