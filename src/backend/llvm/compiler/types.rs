use ahash::{HashMap, HashSet};

use super::memory::AllocatedObject;

pub type AllocatedObjects<'ctx> = &'ctx HashMap<&'ctx str, AllocatedObject<'ctx>>;
pub type MappedHeapPointers<'ctx> = HashSet<(&'ctx str, u32, bool)>;
pub type MappedHeapPointer<'ctx> = (&'ctx str, u32, bool);
