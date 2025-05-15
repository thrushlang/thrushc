use {
    super::{
        super::super::super::logging::{self, LoggingType},
        memory::{self, SymbolAllocated},
        typegen,
        types::{ScopeCall, ScopeCalls, SymbolsAllocated},
        valuegen,
    },
    crate::{
        middle::{
            statement::{Function, ThrushAttributes},
            types::Type,
        },
        standard::diagnostic::Diagnostician,
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

const CONSTANTS_MINIMAL_CAPACITY: usize = 255;
const FUNCTION_MINIMAL_CAPACITY: usize = 255;
const SCOPE_MINIMAL_CAPACITY: usize = 155;
const CALLS_PER_SCOPE_MINIMAL_CAPACITY: usize = 100;

#[derive(Debug)]
pub struct CodeGenContext<'a, 'ctx> {
    module: &'a Module<'ctx>,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    position: CodeGenContextPosition,
    pub target_data: TargetData,
    diagnostician: Diagnostician,
    constants: HashMap<&'ctx str, SymbolAllocated<'ctx>>,
    functions: HashMap<&'ctx str, Function<'ctx>>,
    blocks: Vec<HashMap<&'ctx str, SymbolAllocated<'ctx>>>,
    scope_calls: Vec<ScopeCall<'ctx>>,
    lift_instructions: HashMap<&'ctx str, SymbolAllocated<'ctx>>,
    scope: usize,
}

impl<'a, 'ctx> CodeGenContext<'a, 'ctx> {
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
            position: CodeGenContextPosition::default(),
            target_data,
            diagnostician,
            constants: HashMap::with_capacity(CONSTANTS_MINIMAL_CAPACITY),
            functions: HashMap::with_capacity(FUNCTION_MINIMAL_CAPACITY),
            blocks: Vec::with_capacity(SCOPE_MINIMAL_CAPACITY),
            scope_calls: Vec::with_capacity(CALLS_PER_SCOPE_MINIMAL_CAPACITY),
            lift_instructions: HashMap::with_capacity(SCOPE_MINIMAL_CAPACITY),
            scope: 0,
        }
    }

    pub fn alloc_local(&mut self, name: &'ctx str, kind: &'ctx Type) {
        let ptr_allocated: PointerValue = valuegen::alloc(
            self.context,
            self.builder,
            kind,
            kind.is_heap_allocated(self.context, &self.target_data),
        );

        let symbol_allocated: SymbolAllocated = SymbolAllocated::new_local(ptr_allocated, kind);

        self.blocks
            .last_mut()
            .unwrap()
            .insert(name, symbol_allocated);
    }

    pub fn alloc_constant(
        &mut self,
        name: &'ctx str,
        kind: &'ctx Type,
        value: BasicValueEnum<'ctx>,
        attributes: &'ctx ThrushAttributes<'ctx>,
    ) {
        let constant_allocated: PointerValue = valuegen::alloc_constant(
            self.module,
            name,
            typegen::generate_type(self.context, kind),
            value,
            attributes,
        );

        let symbol_allocated: SymbolAllocated =
            SymbolAllocated::new_constant(constant_allocated, kind);

        self.constants.insert(name, symbol_allocated);
    }

    pub fn alloc_function_parameter(
        &mut self,
        name: &'ctx str,
        kind: &'ctx Type,
        is_mutable: bool,
        mut value: BasicValueEnum<'ctx>,
    ) {
        if is_mutable && !value.is_pointer_value() && !kind.is_mut_type() {
            let new_value: PointerValue = valuegen::alloc(
                self.context,
                self.builder,
                kind,
                kind.is_heap_allocated(self.context, &self.target_data),
            );

            memory::store_anon(self.builder, new_value, value);

            value = new_value.into();
        }

        let symbol_allocated: SymbolAllocated = SymbolAllocated::new_parameter(value, kind);

        self.lift_instructions.insert(name, symbol_allocated);
    }

    #[inline]
    pub fn insert_function(&mut self, name: &'ctx str, function: Function<'ctx>) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn get_allocated_symbols(&self) -> SymbolsAllocated {
        self.blocks.last().unwrap()
    }

    #[inline]
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

    #[inline]
    pub fn get_function(&self, name: &str) -> Function<'ctx> {
        if let Some(function) = self.functions.get(name) {
            return *function;
        }

        logging::log(
            LoggingType::Panic,
            &format!("Unable to get '{}' function in global frame.", name),
        );

        unreachable!()
    }

    pub fn get_llvm_module(&self) -> &'a Module<'ctx> {
        self.module
    }

    pub fn get_llvm_context(&self) -> &'ctx Context {
        self.context
    }

    pub fn get_llvm_builder(&self) -> &'ctx Builder<'ctx> {
        self.builder
    }

    pub fn set_position(&mut self, new_position: CodeGenContextPosition) {
        self.position = new_position;
    }

    pub fn get_position(&self) -> CodeGenContextPosition {
        self.position
    }

    pub fn set_position_irrelevant(&mut self) {
        self.position = CodeGenContextPosition::NoRelevant;
    }

    pub fn get_diagnostician(&self) -> &Diagnostician {
        &self.diagnostician
    }

    pub fn get_scope_calls(&self) -> &ScopeCalls<'ctx> {
        &self.scope_calls
    }

    pub fn add_scope_call(&mut self, call: ScopeCall<'ctx>) {
        self.scope_calls.push(call);
    }

    pub fn begin_scope(&mut self) {
        self.blocks
            .push(HashMap::with_capacity(SCOPE_MINIMAL_CAPACITY));
        self.blocks
            .last_mut()
            .unwrap()
            .extend(self.lift_instructions.clone());
        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.blocks.pop();

        self.lift_instructions.clear();
        self.scope_calls.clear();

        self.scope -= 1;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum CodeGenContextPosition {
    Local,
    Call,
    Mutation,

    #[default]
    NoRelevant,
}

impl CodeGenContextPosition {
    pub fn in_local(&self) -> bool {
        matches!(self, CodeGenContextPosition::Local)
    }

    pub fn in_call(&self) -> bool {
        matches!(self, CodeGenContextPosition::Call)
    }

    pub fn in_mutation(&self) -> bool {
        matches!(self, CodeGenContextPosition::Mutation)
    }
}
