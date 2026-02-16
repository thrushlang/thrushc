use ahash::AHashMap as HashMap;

use crate::{
    memory::SymbolAllocated,
    traits::LLVMFunctionExtensions,
    types::{
        LLVMFunction, LLVMFunctions, LLVMFunctionsParameters, LLVMGlobalConstants,
        LLVMGlobalStatics, LLVMInstructions, LLVMLocalConstants, LLVMLocalStatics,
    },
};

#[derive(Debug)]
pub struct LLVMSymbolsTable<'ctx> {
    functions: LLVMFunctions<'ctx>,

    global_constants: LLVMGlobalConstants<'ctx>,
    global_statics: LLVMGlobalStatics<'ctx>,

    local_statics: LLVMLocalStatics<'ctx>,
    local_constants: LLVMLocalConstants<'ctx>,

    locals: LLVMInstructions<'ctx>,
    parameters: LLVMFunctionsParameters<'ctx>,

    scope: usize,
}

impl LLVMSymbolsTable<'_> {
    #[inline]
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(1000),
            global_statics: HashMap::with_capacity(1000),
            local_statics: Vec::with_capacity(255),
            global_constants: HashMap::with_capacity(1000),
            local_constants: Vec::with_capacity(255),
            locals: Vec::with_capacity(255),

            parameters: HashMap::with_capacity(15),

            scope: 0,
        }
    }
}

impl<'ctx> LLVMSymbolsTable<'ctx> {
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
            return SymbolAllocated::new_function(
                function.get_value().as_global_value().as_pointer_value(),
                function.get_span(),
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

impl<'ctx> LLVMSymbolsTable<'ctx> {
    #[inline]
    pub fn add_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn add_parameter(&mut self, name: &'ctx str, parameter: SymbolAllocated<'ctx>) {
        self.parameters.insert(name, parameter);
    }

    #[inline]
    pub fn add_global_constant(&mut self, name: &'ctx str, constant: SymbolAllocated<'ctx>) {
        self.global_constants.insert(name, constant);
    }

    #[inline]
    pub fn add_global_static(&mut self, name: &'ctx str, static_: SymbolAllocated<'ctx>) {
        self.global_statics.insert(name, static_);
    }
}

impl<'ctx> LLVMSymbolsTable<'ctx> {
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
}

impl LLVMSymbolsTable<'_> {
    pub fn begin_scope(&mut self) {
        self.local_statics.push(HashMap::with_capacity(255));
        self.local_constants.push(HashMap::with_capacity(255));
        self.locals.push(HashMap::with_capacity(255));

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
fn codegen_abort<T: std::fmt::Display>(message: T) -> ! {
    thrustc_logging::print_backend_bug(
        thrustc_logging::LoggingType::BackendBug,
        &format!("{}", message),
    );
}
