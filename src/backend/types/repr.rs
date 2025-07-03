use ahash::AHashMap as HashMap;
use inkwell::{types::FunctionType, values::FunctionValue};

use crate::{backend::llvm::compiler::memory::SymbolAllocated, frontend::types::lexer::Type};

pub type LLVMGlobalConstants<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalConstants<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [Type], u32);
pub type LLVMFunctions<'ctx> = HashMap<&'ctx str, LLVMFunction<'ctx>>;

pub type LLVMInstructions<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;
pub type LLVMFunctionsParameters<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMInstrinsic<'ctx> = (&'ctx str, FunctionType<'ctx>);
