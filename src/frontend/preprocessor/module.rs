use crate::{
    core::compiler::options::CompilationUnit, frontend::preprocessor::types::ExternalSymbols,
};

#[derive(Debug)]
pub struct Module<'module> {
    file: CompilationUnit,
    symbols: ExternalSymbols<'module>,
}

impl<'module> Module<'module> {
    #[inline]
    pub fn new(file: CompilationUnit) -> Self {
        Self {
            file,
            symbols: Vec::with_capacity(100_000),
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
    pub fn merge(&mut self, other: Module<'module>) {
        self.symbols.extend(other.symbols);
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

    pub fn get_unit(&self) -> &CompilationUnit {
        &self.file
    }
}
