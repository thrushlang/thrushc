use ahash::AHashMap as HashMap;

use crate::frontend::types::semantic::typechecker::types::{
    TypeCheckerAssemblerFunction, TypeCheckerAssemblerFunctions, TypeCheckerFunction,
    TypeCheckerFunctions, TypeCheckerLLI, TypeCheckerLLIs, TypeCheckerLocal, TypeCheckerLocals,
};

#[derive(Debug)]
pub struct TypeCheckerSymbolsTable<'symbol> {
    functions: TypeCheckerFunctions<'symbol>,
    asm_functions: TypeCheckerAssemblerFunctions<'symbol>,
    locals: TypeCheckerLocals<'symbol>,
    llis: TypeCheckerLLIs<'symbol>,
    scope: usize,
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(100),
            asm_functions: HashMap::with_capacity(100),
            locals: Vec::with_capacity(255),
            llis: Vec::with_capacity(255),
            scope: 0,
        }
    }

    pub fn new_local(&mut self, name: &'symbol str, local: TypeCheckerLocal<'symbol>) {
        self.locals.last_mut().unwrap().insert(name, local);
    }

    pub fn new_lli(&mut self, name: &'symbol str, lli: TypeCheckerLLI<'symbol>) {
        self.llis.last_mut().unwrap().insert(name, lli);
    }

    pub fn new_asm_function(
        &mut self,
        name: &'symbol str,
        function: TypeCheckerAssemblerFunction<'symbol>,
    ) {
        self.asm_functions.insert(name, function);
    }

    pub fn new_function(&mut self, name: &'symbol str, function: TypeCheckerFunction<'symbol>) {
        self.functions.insert(name, function);
    }

    pub fn get_function(&self, name: &'symbol str) -> Option<&TypeCheckerFunction<'symbol>> {
        self.functions.get(name)
    }

    pub fn get_asm_function(
        &self,
        name: &'symbol str,
    ) -> Option<&TypeCheckerAssemblerFunction<'symbol>> {
        self.asm_functions.get(name)
    }

    pub fn begin_scope(&mut self) {
        self.llis.push(HashMap::with_capacity(255));
        self.locals.push(HashMap::with_capacity(255));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.llis.pop();
        self.locals.pop();

        self.scope -= 1;
    }
}
