use ahash::AHashMap as HashMap;
use inkwell::values::FunctionValue;

use crate::backend::llvm::compiler::memory::SymbolAllocated;
use crate::frontend::{lexer::span::Span, typesystem::types::Type};

pub type LLVMGlobalConstants<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalConstants<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMGlobalStatics<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalStatics<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [Type], u32, Span);
pub type LLVMFunctions<'ctx> = HashMap<&'ctx str, LLVMFunction<'ctx>>;

pub type LLVMInstructions<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;
pub type LLVMFunctionsParameters<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
