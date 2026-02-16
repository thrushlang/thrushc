use ahash::AHashMap as HashMap;

use thrustc_entities::typechecker::*;

#[derive(Debug)]
pub struct TypeCheckerSymbolsTable<'symbol> {
    functions: TypeCheckerFunctions<'symbol>,
    asm_functions: TypeCheckerAssemblerFunctions<'symbol>,
    intrinsics: TypeCheckerIntrinsics<'symbol>,

    locals: TypeCheckerLocals<'symbol>,

    scope: usize,
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    #[inline]
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(1000),
            asm_functions: HashMap::with_capacity(1000),
            intrinsics: HashMap::with_capacity(1000),

            locals: Vec::with_capacity(255),

            scope: 0,
        }
    }
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    #[inline]
    pub fn new_local(&mut self, name: &'symbol str, local: TypeCheckerLocal<'symbol>) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, local);
        }
    }

    #[inline]
    pub fn new_asm_function(
        &mut self,
        name: &'symbol str,
        function: TypeCheckerAssemblerFunction<'symbol>,
    ) {
        self.asm_functions.insert(name, function);
    }

    #[inline]
    pub fn new_function(&mut self, name: &'symbol str, function: TypeCheckerFunction<'symbol>) {
        self.functions.insert(name, function);
    }

    #[inline]
    pub fn new_intrinsic(&mut self, name: &'symbol str, intrinsic: TypeCheckerIntrinsic<'symbol>) {
        self.intrinsics.insert(name, intrinsic);
    }
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    #[inline]
    pub fn get_function(&self, name: &'symbol str) -> Option<&TypeCheckerFunction<'symbol>> {
        self.functions.get(name)
    }

    #[inline]
    pub fn get_asm_function(
        &self,
        name: &'symbol str,
    ) -> Option<&TypeCheckerAssemblerFunction<'symbol>> {
        self.asm_functions.get(name)
    }

    #[inline]
    pub fn get_intrinsic(&self, name: &'symbol str) -> Option<&TypeCheckerIntrinsic<'symbol>> {
        self.intrinsics.get(name)
    }
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    #[inline]
    pub fn constains_function(&self, name: &'symbol str) -> bool {
        self.functions.contains_key(name)
    }

    #[inline]
    pub fn constains_asm_function(&self, name: &'symbol str) -> bool {
        self.asm_functions.contains_key(name)
    }

    #[inline]
    pub fn constains_intrinsic(&self, name: &'symbol str) -> bool {
        self.intrinsics.contains_key(name)
    }
}

impl TypeCheckerSymbolsTable<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.locals.push(HashMap::with_capacity(255));

        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.locals.pop();

        self.scope -= 1;
    }
}
