use ahash::AHashMap as HashMap;

use crate::frontend::types::typeresolver::types::{TypeResolverFunction, TypeResolverFunctions};

#[derive(Debug)]
pub struct TypeResolverSymbols<'typer> {
    functions: TypeResolverFunctions<'typer>,
}

impl<'typer> TypeResolverSymbols<'typer> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(255),
        }
    }

    pub fn new_function(&mut self, name: &'typer str, function: TypeResolverFunction<'typer>) {
        self.functions.insert(name, function);
    }

    pub fn get_function(&mut self, name: &'typer str) -> Option<&mut TypeResolverFunction<'typer>> {
        self.functions.get_mut(name)
    }
}
