use ahash::AHashMap as HashMap;
use inkwell::values::BasicValueEnum;

use crate::middle::types::Type;

use super::memory::SymbolAllocated;

pub type SymbolsAllocated<'ctx> = &'ctx HashMap<&'ctx str, SymbolAllocated<'ctx>>;

pub type ScopeCall<'ctx> = (&'ctx Type, BasicValueEnum<'ctx>);
pub type ScopeCalls<'ctx> = Vec<ScopeCall<'ctx>>;
