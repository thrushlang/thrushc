use ahash::AHashMap as HashMap;

use crate::signatures::Symbol;

#[derive(Debug)]
pub struct ModuleTable<'module> {
    types: HashMap<&'module str, Symbol>,
    functions: HashMap<&'module str, Symbol>,
    constants: HashMap<&'module str, Symbol>,
    statics: HashMap<&'module str, Symbol>,
    structs: HashMap<&'module str, Symbol>,
    enums: HashMap<&'module str, Symbol>,
}

impl ModuleTable<'_> {
    pub fn new() -> Self {
        Self {
            types: HashMap::with_capacity(100),
            functions: HashMap::with_capacity(100),
            constants: HashMap::with_capacity(100),
            statics: HashMap::with_capacity(100),
            structs: HashMap::with_capacity(100),
            enums: HashMap::with_capacity(100),
        }
    }
}

impl<'module> ModuleTable<'module> {
    #[inline]
    pub fn add_function(&mut self, name: &'module str, symbol: Symbol) {
        self.functions.insert(name, symbol);
    }

    #[inline]
    pub fn add_constant(&mut self, name: &'module str, symbol: Symbol) {
        self.constants.insert(name, symbol);
    }

    #[inline]
    pub fn add_static(&mut self, name: &'module str, symbol: Symbol) {
        self.statics.insert(name, symbol);
    }

    #[inline]
    pub fn add_type(&mut self, name: &'module str, symbol: Symbol) {
        self.types.insert(name, symbol);
    }

    #[inline]
    pub fn add_enum(&mut self, name: &'module str, symbol: Symbol) {
        self.enums.insert(name, symbol);
    }

    #[inline]
    pub fn add_struct(&mut self, name: &'module str, symbol: Symbol) {
        self.structs.insert(name, symbol);
    }
}

impl<'module> ModuleTable<'module> {
    #[inline]
    pub fn get_type(&self, name: &'module str) -> Option<&Symbol> {
        self.types.get(name)
    }

    #[inline]
    pub fn get_enum(&self, name: &'module str) -> Option<&Symbol> {
        self.enums.get(name)
    }

    #[inline]
    pub fn get_struct(&self, name: &'module str) -> Option<&Symbol> {
        self.structs.get(name)
    }
}
