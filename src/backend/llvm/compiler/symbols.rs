use std::fmt::Display;

use ahash::AHashMap as HashMap;

use crate::{
    backend::{
        llvm::compiler::memory::SymbolAllocated,
        types::repr::{
            LLVMFunction, LLVMFunctions, LLVMFunctionsParameters, LLVMGlobalConstants,
            LLVMGlobalStatics, LLVMInstructions, LLVMLocalConstants, LLVMLocalStatics,
        },
    },
    core::console::logging::{self, LoggingType},
};

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
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(1000),

            global_statics: HashMap::with_capacity(1000),
            local_statics: Vec::with_capacity(1000),

            global_constants: HashMap::with_capacity(1000),
            local_constants: Vec::with_capacity(1000),

            locals: Vec::with_capacity(1000),
            parameters: HashMap::with_capacity(10),

            scope: 0,
        }
    }
}

impl<'ctx> SymbolsTable<'ctx> {
    pub fn get_symbol(&self, name: &str) -> SymbolAllocated<'ctx> {
        if let Some(parameter) = self.parameters.get(name) {
            return *parameter;
        }

        if let Some(global_constant) = self.global_constants.get(name) {
            return *global_constant;
        }

        for position in (0..self.scope).rev() {
            if let Some(local_constant) = self.local_constants[position].get(name) {
                return *local_constant;
            }
        }

        if let Some(global_static) = self.global_statics.get(name) {
            return *global_static;
        }

        for position in (0..self.scope).rev() {
            if let Some(local_static) = self.local_statics[position].get(name) {
                return *local_static;
            }
        }

        for position in (0..self.scope).rev() {
            if let Some(local) = self.locals[position].get(name) {
                return *local;
            }
        }

        self::codegen_abort(format!(
            "Unable to get '{}' allocated object at frame pointer number #{}.",
            name, self.scope
        ));
    }

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
    pub fn get_mut_functions(&mut self) -> &mut LLVMFunctions<'ctx> {
        &mut self.functions
    }

    pub fn get_mut_global_constants(&mut self) -> &mut LLVMGlobalConstants<'ctx> {
        &mut self.global_constants
    }

    pub fn get_mut_local_constants(&mut self) -> &mut LLVMLocalConstants<'ctx> {
        &mut self.local_constants
    }

    pub fn get_mut_global_statics(&mut self) -> &mut LLVMGlobalStatics<'ctx> {
        &mut self.global_statics
    }

    pub fn get_mut_local_statics(&mut self) -> &mut LLVMLocalStatics<'ctx> {
        &mut self.local_statics
    }

    pub fn get_mut_locals(&mut self) -> &mut LLVMInstructions<'ctx> {
        &mut self.locals
    }

    pub fn get_mut_parameters(&mut self) -> &mut LLVMFunctionsParameters<'ctx> {
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

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
