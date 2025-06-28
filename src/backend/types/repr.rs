use ahash::AHashMap as HashMap;
use inkwell::{types::FunctionType, values::FunctionValue};

use crate::{backend::llvm::compiler::memory::SymbolAllocated, frontend::types::lexer::ThrushType};

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [ThrushType], u32);

pub type LLVMFunctions<'ctx> = HashMap<&'ctx str, LLVMFunction<'ctx>>;
pub type LLVMInstructions<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;
pub type LLVMFunctionsParameters<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMInstrinsic<'ctx> = (&'ctx str, FunctionType<'ctx>);
