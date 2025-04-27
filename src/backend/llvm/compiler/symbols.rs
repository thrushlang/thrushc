use {
    super::{
        super::super::super::logging::{self, LoggingType},
        memory::{AllocatedSymbol, MemoryFlag},
        types::AllocatedSymbols,
        valuegen,
    },
    crate::middle::{statement::Function, types::Type},
    ahash::AHashMap as HashMap,
    inkwell::{builder::Builder, context::Context, values::PointerValue},
};

const CONSTANTS_MINIMAL_CAPACITY: usize = 255;
const FUNCTION_MINIMAL_CAPACITY: usize = 255;

const SCOPE_MINIMAL_CAPACITY: usize = 155;

#[derive(Debug)]
pub struct SymbolsTable<'ctx> {
    pub context: &'ctx Context,
    pub builder: &'ctx Builder<'ctx>,
    pub constants: HashMap<&'ctx str, AllocatedSymbol<'ctx>>,
    pub functions: HashMap<&'ctx str, Function<'ctx>>,
    pub blocks: Vec<HashMap<&'ctx str, AllocatedSymbol<'ctx>>>,
    pub scope: usize,
}

impl<'ctx> SymbolsTable<'ctx> {
    pub fn new(context: &'ctx Context, builder: &'ctx Builder<'ctx>) -> Self {
        Self {
            context,
            builder,
            constants: HashMap::with_capacity(CONSTANTS_MINIMAL_CAPACITY),
            functions: HashMap::with_capacity(FUNCTION_MINIMAL_CAPACITY),
            blocks: Vec::with_capacity(SCOPE_MINIMAL_CAPACITY),
            scope: 0,
        }
    }

    #[inline]
    pub fn alloc(&mut self, name: &'ctx str, kind: &'ctx Type, memory_flags: &[MemoryFlag]) {
        let allocated_pointer: PointerValue =
            valuegen::alloc(self.context, self.builder, kind, kind.is_stack_allocated());

        let allocated_object: AllocatedSymbol =
            AllocatedSymbol::alloc(allocated_pointer, memory_flags, kind);

        self.blocks
            .last_mut()
            .unwrap()
            .insert(name, allocated_object);
    }

    #[inline]
    pub fn insert_constant_object(&mut self, name: &'ctx str, object: AllocatedSymbol<'ctx>) {
        self.constants.insert(name, object);
    }

    #[inline]
    pub fn insert_function(&mut self, name: &'ctx str, function: Function<'ctx>) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn get_allocated_symbols(&self) -> AllocatedSymbols {
        self.blocks.last().unwrap()
    }

    #[inline]
    pub fn get_allocated_symbol(&self, name: &str) -> AllocatedSymbol<'ctx> {
        if let Some(constant) = self.constants.get(name) {
            return *constant;
        }

        for position in (0..self.scope).rev() {
            if let Some(allocated_symbol) = self.blocks[position].get(name) {
                return *allocated_symbol;
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

    #[inline]
    pub fn begin_scope(&mut self) {
        self.blocks
            .push(HashMap::with_capacity(SCOPE_MINIMAL_CAPACITY));
        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.blocks.pop();
        self.scope -= 1;
    }
}
