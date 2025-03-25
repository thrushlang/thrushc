use {
    super::{super::super::logging, Instruction, types::Struct},
    ahash::AHashMap as HashMap,
    inkwell::values::{FunctionValue, PointerValue},
};

#[derive(Debug, Clone)]
pub struct CompilerObjects<'ctx> {
    pub functions: HashMap<&'ctx str, (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>])>,
    pub structs: HashMap<&'ctx str, &'ctx Struct<'ctx>>,
    pub blocks: Vec<HashMap<&'ctx str, PointerValue<'ctx>>>,
    pub scope_position: usize,
}

impl<'ctx> CompilerObjects<'ctx> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(255),
            structs: HashMap::with_capacity(255),
            blocks: Vec::with_capacity(100),
            scope_position: 0,
        }
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.blocks.push(HashMap::new());
        self.scope_position += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.blocks.pop();
        self.scope_position -= 1;
    }

    #[inline]
    pub fn insert(&mut self, name: &'ctx str, value: PointerValue<'ctx>) {
        self.blocks[self.scope_position - 1].insert(name, value);
    }

    #[inline]
    pub fn insert_function(
        &mut self,
        name: &'ctx str,
        function: (FunctionValue<'ctx>, &'ctx [Instruction<'ctx>]),
    ) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn insert_struct(&mut self, name: &'ctx str, fields_types: &'ctx Struct) {
        self.structs.insert(name, fields_types);
    }

    #[inline]
    pub fn get_struct(&self, name: &str) -> Option<&Struct> {
        self.structs.get(name).map(|structure| &**structure)
    }

    #[inline]
    pub fn get_local(&self, name: &str) -> PointerValue<'ctx> {
        for position in (0..self.scope_position).rev() {
            if self.blocks[position].contains_key(name) {
                return *self.blocks[position].get(name).unwrap();
            }
        }

        logging::log(
            logging::LogType::Panic,
            &format!(
                "Unable to get '{}' pointer at frame pointer number #{}.",
                name, self.scope_position
            ),
        );

        unreachable!()
    }

    #[inline]
    pub fn get_function(
        &self,
        name: &str,
    ) -> Option<(FunctionValue<'ctx>, &'ctx [Instruction<'ctx>])> {
        if self.functions.contains_key(name) {
            return Some(*self.functions.get(name).unwrap());
        }

        None
    }
}
