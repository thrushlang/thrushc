use crate::back_end::llvm_codegen::attributes::LLVMAttribute;
use crate::back_end::llvm_codegen::memory::SymbolAllocated;

use crate::core::diagnostic::span::Span;
use crate::front_end::typesystem::types::Type;

use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use inkwell::values::{FunctionValue, PointerValue};

pub type LLVMGlobalConstants<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalConstants<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMGlobalStatics<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;
pub type LLVMLocalStatics<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;

pub type LLVMDBGFunction<'ctx> = (
    String,
    FunctionValue<'ctx>,
    &'ctx Type,
    &'ctx [Type],
    bool,
    bool,
    Span,
);

pub type LLVMFunction<'ctx> = (FunctionValue<'ctx>, &'ctx Type, &'ctx [Type], u32, Span);
pub type LLVMFunctions<'ctx> = HashMap<&'ctx str, LLVMFunction<'ctx>>;

pub type LLVMInstructions<'ctx> = Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>;
pub type LLVMFunctionsParameters<'ctx> = HashMap<&'ctx str, SymbolAllocated<'ctx>>;

pub type LLVMAttributes<'ctx> = Vec<LLVMAttribute<'ctx>>;

pub type LLVMCtors<'ctx> = HashSet<(PointerValue<'ctx>, u32)>;
pub type LLVMDtors<'ctx> = HashSet<(PointerValue<'ctx>, u32)>;
