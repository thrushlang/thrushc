use {
    super::{
        super::super::super::logging::{self, LoggingType},
        memory::{SymbolAllocated, SymbolToAllocate},
        typegen,
    },
    crate::{
        backend::{
            llvm::compiler::{
                alloc::{self},
                memory::LLVMAllocationSite,
            },
            types::repr::{
                LLVMConstants, LLVMFunction, LLVMFunctions, LLVMFunctionsParameters,
                LLVMInstructions,
            },
        },
        core::diagnostic::diagnostician::Diagnostician,
        frontend::types::{lexer::ThrushType, parser::stmts::types::ThrushAttributes},
    },
    ahash::AHashMap as HashMap,
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        targets::TargetData,
        values::{BasicValueEnum, PointerValue},
    },
};

#[derive(Debug)]
pub struct LLVMCodeGenContext<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    target_data: TargetData,

    functions: LLVMFunctions<'ctx>,
    parameters: LLVMFunctionsParameters<'ctx>,
    constants: LLVMConstants<'ctx>,
    instructions: LLVMInstructions<'ctx>,

    scope: usize,

    site_allocation: Option<LLVMAllocationSite>,

    diagnostician: Diagnostician,
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn new(
        module: &'a Module<'ctx>,
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
        target_data: TargetData,
        diagnostician: Diagnostician,
    ) -> Self {
        Self {
            module,
            context,
            builder,
            target_data,
            constants: HashMap::with_capacity(10000),
            functions: HashMap::with_capacity(10000),
            parameters: HashMap::with_capacity(10),
            instructions: Vec::with_capacity(1000),
            scope: 0,
            site_allocation: None,
            diagnostician,
        }
    }
}

impl<'ctx> LLVMCodeGenContext<'_, 'ctx> {
    pub fn alloc_local(
        &mut self,
        name: &'ctx str,
        ascii_name: &'ctx str,
        kind: &'ctx ThrushType,
        attributes: &'ctx ThrushAttributes<'ctx>,
    ) {
        let ptr: PointerValue = alloc::alloc(self, ascii_name, kind, attributes);

        let local: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Local, kind, ptr.into());

        if let Some(last_block) = self.instructions.last_mut() {
            last_block.insert(name, local);
        } else {
            logging::log(
                LoggingType::BackendPanic,
                "The last frame of symbols could not be obtained.",
            );
        }
    }

    pub fn alloc_lli(
        &mut self,
        name: &'ctx str,
        kind: &'ctx ThrushType,
        value: BasicValueEnum<'ctx>,
    ) {
        let lli: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::LowLevelInstruction, kind, value);

        if let Some(last_block) = self.instructions.last_mut() {
            last_block.insert(name, lli);
        } else {
            logging::log(
                LoggingType::BackendPanic,
                "The last frame of symbols could not be obtained.",
            );
        }
    }

    pub fn alloc_constant(
        &mut self,
        name: &'ctx str,
        kind: &'ctx ThrushType,
        value: BasicValueEnum<'ctx>,
        attributes: &'ctx ThrushAttributes<'ctx>,
    ) {
        let ptr: PointerValue = alloc::constant(
            self.module,
            name,
            typegen::generate_type(self.context, kind),
            value,
            attributes,
        );

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Constant, kind, ptr.into());

        self.constants.insert(name, symbol_allocated);
    }

    pub fn alloc_function_parameter(
        &mut self,
        name: &'ctx str,
        kind: &'ctx ThrushType,
        value: BasicValueEnum<'ctx>,
    ) {
        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(SymbolToAllocate::Parameter, kind, value);

        self.parameters.insert(name, symbol_allocated);
    }

    pub fn get_allocated_symbol(&self, name: &str) -> SymbolAllocated<'ctx> {
        if let Some(fn_parameter) = self.parameters.get(name) {
            return *fn_parameter;
        }

        if let Some(constant) = self.constants.get(name) {
            return *constant;
        }

        for position in (0..self.scope).rev() {
            if let Some(instruction) = self.instructions[position].get(name) {
                return *instruction;
            }
        }

        logging::log(
            LoggingType::BackendPanic,
            &format!(
                "Unable to get '{}' allocated object at frame pointer number #{}.",
                name, self.scope
            ),
        );

        unreachable!()
    }

    pub fn get_function(&self, name: &str) -> LLVMFunction<'ctx> {
        if let Some(function) = self.functions.get(name) {
            return *function;
        }

        logging::log(
            LoggingType::BackendPanic,
            &format!("Unable to get '{}' function in global frame.", name),
        );

        unreachable!()
    }

    pub fn add_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.functions.insert(name, function);
    }

    pub fn set_site_allocation(&mut self, site: LLVMAllocationSite) {
        self.site_allocation = Some(site);
    }

    pub fn reset_site_allocation(&mut self) {
        self.site_allocation = None;
    }

    pub fn get_site_allocation(&self) -> Option<LLVMAllocationSite> {
        self.site_allocation
    }

    pub fn begin_scope(&mut self) {
        self.instructions.push(HashMap::with_capacity(256));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.instructions.pop();
        self.scope -= 1;

        if self.scope == 0 {
            self.parameters.clear();
        }
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    #[inline]
    pub fn get_llvm_module(&self) -> &'a Module<'ctx> {
        self.module
    }

    #[inline]
    pub fn get_llvm_context(&self) -> &'ctx Context {
        self.context
    }

    #[inline]
    pub fn get_llvm_builder(&self) -> &'ctx Builder<'ctx> {
        self.builder
    }

    #[inline]
    pub fn get_target_data(&self) -> &TargetData {
        &self.target_data
    }

    #[inline]
    pub fn get_diagnostician(&self) -> &Diagnostician {
        &self.diagnostician
    }
}
