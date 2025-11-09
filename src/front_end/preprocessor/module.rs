use crate::{
    core::compiler::options::CompilationUnit, front_end::preprocessor::types::ExternalSymbols,
};

#[derive(Debug)]
pub struct Module<'module> {
    file: CompilationUnit,
    symbols: ExternalSymbols<'module>,
    submodules: Vec<Module<'module>>,
    id: usize,
}

impl<'module> Module<'module> {
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

impl<'module> Module<'module> {
    #[inline]
    pub fn append_symbols(&mut self, symbols: &mut ExternalSymbols<'module>) {
        self.symbols.append(symbols);
    }
}

impl<'module> Module<'module> {
    #[inline]
    pub fn add_submodule(&mut self, other: Module<'module>) {
        self.submodules.push(other);
    }
}

impl<'module> Module<'module> {
    #[inline]
    pub fn get_mut_symbols(&mut self) -> &mut ExternalSymbols<'module> {
        &mut self.symbols
    }
}

impl<'module> Module<'module> {
    #[inline]
    pub fn get_symbols(&self) -> &ExternalSymbols<'module> {
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
