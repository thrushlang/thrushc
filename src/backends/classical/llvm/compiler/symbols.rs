use crate::backends::classical::llvm::compiler::constants::LLVM_COMPILER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY;
use crate::backends::classical::llvm::compiler::constants::LLVM_COMPILER_SYMBOLS_LOCAL_MINIMAL_CAPACITY;
use crate::backends::classical::llvm::compiler::memory::SymbolAllocated;
use crate::backends::classical::types::repr::LLVMFunction;
use crate::backends::classical::types::repr::LLVMFunctions;
use crate::backends::classical::types::repr::LLVMFunctionsParameters;
use crate::backends::classical::types::repr::LLVMGlobalConstants;
use crate::backends::classical::types::repr::LLVMGlobalStatics;
use crate::backends::classical::types::repr::LLVMInstructions;
use crate::backends::classical::types::repr::LLVMLocalConstants;
use crate::backends::classical::types::repr::LLVMLocalStatics;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use ahash::AHashMap as HashMap;
use inkwell::values::FunctionValue;
use std::fmt::Display;

#[derive(Debug)]
pub struct SymbolsTable<'ctx> {
    functions: LLVMFunctions<'ctx>,
    global_statics: LLVMGlobalStatics<'ctx>,
    local_statics: LLVMLocalStatics<'ctx>,
    global_constants: LLVMGlobalConstants<'ctx>,
    local_constants: LLVMLocalConstants<'ctx>,
    locals: LLVMInstructions<'ctx>,
    parameters: LLVMFunctionsParameters<'ctx>,

    scope: usize,
}

impl SymbolsTable<'_> {
    #[inline]
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(LLVM_COMPILER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY),
            global_statics: HashMap::with_capacity(LLVM_COMPILER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY),
            local_statics: Vec::with_capacity(LLVM_COMPILER_SYMBOLS_LOCAL_MINIMAL_CAPACITY),
            global_constants: HashMap::with_capacity(LLVM_COMPILER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY),
            local_constants: Vec::with_capacity(LLVM_COMPILER_SYMBOLS_LOCAL_MINIMAL_CAPACITY),
            locals: Vec::with_capacity(LLVM_COMPILER_SYMBOLS_LOCAL_MINIMAL_CAPACITY),

            parameters: HashMap::with_capacity(15),

            scope: 0,
        }
    }
}

impl<'ctx> SymbolsTable<'ctx> {
    #[must_use]
    pub fn get_symbol(&self, name: &str) -> SymbolAllocated<'ctx> {
        if let Some(parameter) = self.parameters.get(name) {
            return *parameter;
        }

        for position in (0..self.scope).rev() {
            if let Some(scope) = self.locals.get(position) {
                if let Some(local) = scope.get(name) {
                    return *local;
                }
            }
        }

        if let Some(global_constant) = self.global_constants.get(name) {
            return *global_constant;
        }
        for position in (0..self.scope).rev() {
            if let Some(scope) = self.local_constants.get(position) {
                if let Some(local_constant) = scope.get(name) {
                    return *local_constant;
                }
            }
        }

        if let Some(global_static) = self.global_statics.get(name) {
            return *global_static;
        }
        for position in (0..self.scope).rev() {
            if let Some(scope) = self.local_statics.get(position) {
                if let Some(local_static) = scope.get(name) {
                    return *local_static;
                }
            }
        }

        if let Some(function) = self.functions.get(name) {
            let llvm_function: FunctionValue = function.0;

            return SymbolAllocated::new_function(
                llvm_function.as_global_value().as_pointer_value(),
            );
        }

        self::codegen_abort(format!(
            "Unable to get '{}' allocated object at frame pointer number '#{}'.",
            name, self.scope
        ));
    }

    #[must_use]
    pub fn get_function(&self, name: &str) -> LLVMFunction<'ctx> {
        if let Some(function) = self.functions.get(name) {
            return *function;
        }

        self::codegen_abort(format!(
            "Unable to get '{}' function in global frame.",
            name
        ));
    }
}

impl<'ctx> SymbolsTable<'ctx> {
    #[inline]
    pub fn get_mut_all_functions(&mut self) -> &mut LLVMFunctions<'ctx> {
        &mut self.functions
    }

    #[inline]
    pub fn get_mut_all_global_constants(&mut self) -> &mut LLVMGlobalConstants<'ctx> {
        &mut self.global_constants
    }

    #[inline]
    pub fn get_mut_all_local_constants(&mut self) -> &mut LLVMLocalConstants<'ctx> {
        &mut self.local_constants
    }

    #[inline]
    pub fn get_mut_all_global_statics(&mut self) -> &mut LLVMGlobalStatics<'ctx> {
        &mut self.global_statics
    }

    #[inline]
    pub fn get_mut_all_local_statics(&mut self) -> &mut LLVMLocalStatics<'ctx> {
        &mut self.local_statics
    }

    #[inline]
    pub fn get_mut_all_locals(&mut self) -> &mut LLVMInstructions<'ctx> {
        &mut self.locals
    }

    #[inline]
    pub fn get_mut_all_parameters(&mut self) -> &mut LLVMFunctionsParameters<'ctx> {
        &mut self.parameters
    }
}

impl SymbolsTable<'_> {
    pub fn begin_scope(&mut self) {
        self.local_statics.push(HashMap::with_capacity(256));
        self.local_constants.push(HashMap::with_capacity(256));
        self.locals.push(HashMap::with_capacity(256));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.local_statics.pop();
        self.local_constants.pop();
        self.locals.pop();

        self.scope -= 1;

        if self.scope == 0 {
            self.parameters.clear();
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
