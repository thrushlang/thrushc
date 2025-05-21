use {
    super::{
        super::super::super::logging::{self, LoggingType},
        memory::{SymbolAllocated, SymbolToAllocate},
        typegen, valuegen,
    },
    crate::{
        standard::diagnostic::Diagnostician,
        types::{
            backend::llvm::types::{LLVMFunction, LLVMScopeCall, LLVMScopeCalls, SymbolsAllocated},
            frontend::{lexer::types::ThrushType, parser::stmts::types::CompilerAttributes},
        },
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
    position: LLVMCodeGenContextPosition,
    previous_position: LLVMCodeGenContextPosition,
    pub target_data: TargetData,
    diagnostician: Diagnostician,
    constants: HashMap<&'ctx str, SymbolAllocated<'ctx>>,
    functions: HashMap<&'ctx str, LLVMFunction<'ctx>>,
    blocks: Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>,
    llvm_calls: LLVMScopeCalls<'ctx>,
    lift_instructions: HashMap<&'ctx str, SymbolAllocated<'ctx>>,
    scope: usize,
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
            position: LLVMCodeGenContextPosition::default(),
            previous_position: LLVMCodeGenContextPosition::default(),
            target_data,
            diagnostician,
            constants: HashMap::with_capacity(100),
            functions: HashMap::with_capacity(100),
            blocks: Vec::with_capacity(255),
            llvm_calls: Vec::with_capacity(100),
            lift_instructions: HashMap::with_capacity(100),
            scope: 0,
        }
    }

    pub fn alloc_local(&mut self, name: &'ctx str, kind: &'ctx ThrushType) {
        let ptr_allocated: PointerValue = valuegen::alloc(
            self.context,
            self.builder,
            kind,
            kind.is_heap_allocated(self.context, &self.target_data),
        );

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(self, SymbolToAllocate::Local, ptr_allocated.into(), kind);

        self.blocks
            .last_mut()
            .unwrap()
            .insert(name, symbol_allocated);
    }

    pub fn alloc_constant(
        &mut self,
        name: &'ctx str,
        kind: &'ctx ThrushType,
        value: BasicValueEnum<'ctx>,
        attributes: &'ctx CompilerAttributes<'ctx>,
    ) {
        let ptr_allocated: PointerValue = valuegen::alloc_constant(
            self.module,
            name,
            typegen::generate_type(self.context, kind),
            value,
            attributes,
        );

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(self, SymbolToAllocate::Constant, ptr_allocated.into(), kind);

        self.constants.insert(name, symbol_allocated);
    }

    pub fn alloc_function_parameter(
        &mut self,
        name: &'ctx str,
        kind: &'ctx ThrushType,
        value: BasicValueEnum<'ctx>,
    ) {
        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new(self, SymbolToAllocate::Parameter, value, kind);

        self.lift_instructions.insert(name, symbol_allocated);
    }

    pub fn add_function(&mut self, name: &'ctx str, function: LLVMFunction<'ctx>) {
        self.functions.insert(name, function);
    }

    pub fn get_allocated_symbols(&self) -> SymbolsAllocated {
        self.blocks.last().cloned().unwrap()
    }

    pub fn get_allocated_symbol(&self, name: &str) -> SymbolAllocated<'ctx> {
        if let Some(constant) = self.constants.get(name) {
            return constant.clone();
        }

        for position in (0..self.scope).rev() {
            if let Some(allocated_symbol) = self.blocks[position].get(name) {
                return allocated_symbol.clone();
            }
        }

        logging::log(
            LoggingType::Panic,
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
            LoggingType::Panic,
            &format!("Unable to get '{}' function in global frame.", name),
        );

        unreachable!()
    }

    pub fn begin_scope(&mut self) {
        self.blocks.push(HashMap::with_capacity(256));

        self.blocks
            .last_mut()
            .unwrap()
            .extend(self.lift_instructions.clone());

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.blocks.pop();

        self.lift_instructions.clear();
        self.llvm_calls.clear();

        self.scope -= 1;
    }
}

impl<'a, 'ctx> LLVMCodeGenContext<'a, 'ctx> {
    pub fn get_llvm_module(&self) -> &'a Module<'ctx> {
        self.module
    }

    pub fn get_llvm_context(&self) -> &'ctx Context {
        self.context
    }

    pub fn get_llvm_builder(&self) -> &'ctx Builder<'ctx> {
        self.builder
    }

    pub fn set_position(&mut self, new_position: LLVMCodeGenContextPosition) {
        self.previous_position = self.position;
        self.position = new_position;
    }

    pub fn get_position(&self) -> LLVMCodeGenContextPosition {
        self.position
    }

    pub fn get_previous_position(&self) -> LLVMCodeGenContextPosition {
        self.previous_position
    }

    pub fn set_position_irrelevant(&mut self) {
        self.position = LLVMCodeGenContextPosition::NoRelevant;
    }

    pub fn get_diagnostician(&self) -> &Diagnostician {
        &self.diagnostician
    }

    pub fn get_llvm_calls(&self) -> &LLVMScopeCalls<'ctx> {
        &self.llvm_calls
    }

    pub fn add_scope_call(&mut self, call: LLVMScopeCall<'ctx>) {
        self.llvm_calls.push(call);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum LLVMCodeGenContextPosition {
    Local,
    Call,
    Mutation,

    #[default]
    NoRelevant,
}

impl LLVMCodeGenContextPosition {
    pub fn in_local(&self) -> bool {
        matches!(self, LLVMCodeGenContextPosition::Local)
    }

    pub fn in_call(&self) -> bool {
        matches!(self, LLVMCodeGenContextPosition::Call)
    }

    pub fn in_mutation(&self) -> bool {
        matches!(self, LLVMCodeGenContextPosition::Mutation)
    }
}
