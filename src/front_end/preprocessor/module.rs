use crate::{
    core::compiler::options::CompilationUnit, front_end::preprocessor::types::ExternalSymbols,
};

#[derive(Debug)]
pub struct Module {
    file: CompilationUnit,
    symbols: ExternalSymbols,
    submodules: Vec<Module>,
    id: usize,
}

impl Module {
    #[inline]
    pub fn new(file: CompilationUnit) -> Self {
        Self {
            file,
            symbols: Vec::with_capacity(100_000),
            submodules: Vec::with_capacity(100),
            id: fastrand::usize(1000..10000_00000),
        }
    }
}

impl Module {
    #[inline]
    pub fn append_symbols(&mut self, symbols: &mut ExternalSymbols) {
        self.symbols.append(symbols);
    }
}

impl Module {
    #[inline]
    pub fn add_submodule(&mut self, other: Module) {
        self.submodules.push(other);
    }
}

impl Module {
    #[inline]
    pub fn get_mut_symbols(&mut self) -> &mut ExternalSymbols {
        &mut self.symbols
    }
}

impl Module {
    #[inline]
    pub fn get_symbols(&self) -> &ExternalSymbols {
        &self.symbols
    }

    #[inline]
    pub fn get_unit(&self) -> &CompilationUnit {
        &self.file
    }

    #[inline]
    pub fn get_id(&self) -> usize {
        self.id
    }
}
