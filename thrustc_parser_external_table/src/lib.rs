use thrustc_preprocessor::{module::Module, signatures::Symbol};

#[derive(Debug)]
pub struct ExternalSymbolTable<'parser> {
    modules: &'parser [Module],
}

impl<'parser> ExternalSymbolTable<'parser> {
    #[inline]
    pub fn new(modules: &'parser [Module]) -> Self {
        Self { modules }
    }
}

impl<'parser> ExternalSymbolTable<'parser> {
    pub fn search_symbol_in_first_level(&self) -> Option<&Symbol> {
        None
    }
}
