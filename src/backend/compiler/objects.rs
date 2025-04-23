use {
    super::{
        super::super::logging::{self, LoggingType},
        memory::AllocatedObject,
        types::{AllocatedObjects, Function, Structure},
    },
    ahash::AHashMap as HashMap,
};

const STRUCTURE_MINIMAL_CAPACITY: usize = 255;
const FUNCTION_MINIMAL_CAPACITY: usize = 255;

const SCOPE_MINIMAL_CAPACITY: usize = 155;

#[derive(Debug)]
pub struct CompilerObjects<'ctx> {
    pub structs: HashMap<&'ctx str, Structure<'ctx>>,
    pub functions: HashMap<&'ctx str, Function<'ctx>>,
    pub blocks: Vec<HashMap<&'ctx str, AllocatedObject<'ctx>>>,
    pub scope_position: usize,
}

impl<'ctx> CompilerObjects<'ctx> {
    pub fn new() -> Self {
        Self {
            structs: HashMap::with_capacity(STRUCTURE_MINIMAL_CAPACITY),
            functions: HashMap::with_capacity(FUNCTION_MINIMAL_CAPACITY),
            blocks: Vec::with_capacity(SCOPE_MINIMAL_CAPACITY),
            scope_position: 0,
        }
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.blocks
            .push(HashMap::with_capacity(SCOPE_MINIMAL_CAPACITY));
        self.scope_position += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.blocks.pop();
        self.scope_position -= 1;
    }

    #[inline]
    pub fn alloc_local_object(&mut self, name: &'ctx str, object: AllocatedObject<'ctx>) {
        self.blocks[self.scope_position - 1].insert(name, object);
    }

    #[inline]
    pub fn insert_function(&mut self, name: &'ctx str, function: Function<'ctx>) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn insert_structure(&mut self, name: &'ctx str, structure: Structure<'ctx>) {
        self.structs.insert(name, structure);
    }

    #[inline]
    pub fn get_struct(&self, name: &str) -> &Structure {
        self.structs.get(name).unwrap()
    }

    #[inline]
    pub fn get_allocated_objects(&self) -> AllocatedObjects {
        &self.blocks[self.scope_position - 1]
    }

    #[inline]
    pub fn get_allocated_object(&self, name: &str) -> AllocatedObject<'ctx> {
        for position in (0..self.scope_position).rev() {
            if let Some(allocated_object) = self.blocks[position].get(name) {
                return *allocated_object;
            }
        }

        logging::log(
            LoggingType::Panic,
            &format!(
                "Unable to get '{}' allocated object at frame pointer number #{}.",
                name, self.scope_position
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
}
