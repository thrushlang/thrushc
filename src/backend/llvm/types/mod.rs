use inkwell::values::{FunctionValue, GlobalValue};

use crate::frontend::typesystem::types::Type;

pub mod impls;
pub mod repr;
pub mod traits;

pub type LLVMJITCompilerGlobals<'ctx> = Vec<(GlobalValue<'ctx>, String)>;
pub type LLVMJITCompilerFunctions<'ctx> = Vec<(FunctionValue<'ctx>, String)>;

pub type LLVMGEPIndexes<'ctx> = &'ctx [(Type, u32)];
