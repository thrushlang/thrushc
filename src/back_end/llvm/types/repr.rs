use crate::back_end::llvm::compiler::attributes::LLVMAttribute;
use crate::back_end::llvm::compiler::memory::SymbolAllocated;

use crate::front_end::lexer::span::Span;
use crate::front_end::typesystem::types::Type;

use ahash::AHashMap as HashMap;
use inkwell::values::FunctionValue;

pub type LLVMGlobalConstants<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalConstants<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMGlobalStatics<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalStatics<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx [Type], u32, Span);
pub type LLVMFunctions<'ctx> = HashMap<&'ctx str, LLVMFunction<'ctx>>;

pub type LLVMInstructions<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;
pub type LLVMFunctionsParameters<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;

pub type LLVMAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;
