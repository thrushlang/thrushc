use ahash::AHashMap as HashMap;

use crate::front_end::semantic::typechecker::constants::TYPECHECKER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY;
use crate::front_end::semantic::typechecker::constants::TYPECHECKER_SYMBOLS_LOCAL_MINIMAL_CAPACITY;

use crate::front_end::types::semantic::typechecker::types::TypeCheckerAssemblerFunction;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerAssemblerFunctions;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerFunction;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerFunctions;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerIntrinsic;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerIntrinsics;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerLLI;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerLLIs;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerLocal;
use crate::front_end::types::semantic::typechecker::types::TypeCheckerLocals;

#[derive(Debug)]
pub struct TypeCheckerSymbolsTable<'symbol> {
    functions: TypeCheckerFunctions<'symbol>,
    asm_functions: TypeCheckerAssemblerFunctions<'symbol>,
    intrinsics: TypeCheckerIntrinsics<'symbol>,

    locals: TypeCheckerLocals<'symbol>,
    llis: TypeCheckerLLIs<'symbol>,

    scope: usize,
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    #[inline]
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(TYPECHECKER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY),
            asm_functions: HashMap::with_capacity(TYPECHECKER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY),
            intrinsics: HashMap::with_capacity(TYPECHECKER_SYMBOLS_GLOBAL_MINIMAL_CAPACITY),

            locals: Vec::with_capacity(TYPECHECKER_SYMBOLS_LOCAL_MINIMAL_CAPACITY),
            llis: Vec::with_capacity(TYPECHECKER_SYMBOLS_LOCAL_MINIMAL_CAPACITY),

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
    pub fn new_lli(&mut self, name: &'symbol str, lli: TypeCheckerLLI<'symbol>) {
        if let Some(scope) = self.llis.last_mut() {
            scope.insert(name, lli);
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

impl TypeCheckerSymbolsTable<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.llis.push(HashMap::with_capacity(
            TYPECHECKER_SYMBOLS_LOCAL_MINIMAL_CAPACITY,
        ));

        self.locals.push(HashMap::with_capacity(
            TYPECHECKER_SYMBOLS_LOCAL_MINIMAL_CAPACITY,
        ));

        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.llis.pop();
        self.locals.pop();

        self.scope -= 1;
    }
}
