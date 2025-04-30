use ahash::AHashMap as HashMap;

use super::memory::SymbolAllocated;

pub type SymbolsAllocated<'ctx> = &'ctx HashMap<&'ctx str, SymbolAllocated<'ctx>>;
