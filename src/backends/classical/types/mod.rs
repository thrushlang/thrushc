use crate::frontends::classical::typesystem::types::Type;

pub mod impls;
pub mod repr;
pub mod traits;

pub type LLVMGEPIndexes<'ctx> = &'ctx [(Type, u32)];
