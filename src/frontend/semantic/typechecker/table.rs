use ahash::AHashMap as HashMap;

use crate::types::frontend::{
    lexer::types::ThrushType,
    typechecker::types::{
        TypeCheckerAllMethods, TypeCheckerFunction, TypeCheckerFunctions, TypeCheckerLocal,
        TypeCheckerLocals, TypeCheckerMethod, TypeCheckerMethods,
    },
};

#[derive(Debug)]
pub struct TypeCheckerSymbolsTable<'symbol> {
    functions: TypeCheckerFunctions<'symbol>,
    locals: TypeCheckerLocals<'symbol>,
    methods: TypeCheckerMethods<'symbol>,
    scope: usize,
}

impl<'symbol> TypeCheckerSymbolsTable<'symbol> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(100),
            locals: Vec::with_capacity(255),
            methods: HashMap::with_capacity(100),
            scope: 0,
        }
    }

    pub fn new_local(&mut self, name: &'symbol str, local: TypeCheckerLocal<'symbol>) {
        self.locals.last_mut().unwrap().insert(name, local);
    }

    pub fn new_function(&mut self, name: &'symbol str, function: (&'symbol [ThrushType], bool)) {
        self.functions.insert(name, function);
    }

    pub fn new_methods(&mut self, name: &'symbol str, methods: TypeCheckerAllMethods<'symbol>) {
        self.methods.insert(name, methods);
    }

    pub fn get_local(&self, name: &'symbol str) -> Option<TypeCheckerLocal<'symbol>> {
        for scope in (0..=self.scope - 1).rev() {
            if let Some(scope) = self.locals.get(scope) {
                if let Some(local) = scope.get(name) {
                    return Some(local);
                }
            }
        }

        None
    }

    pub fn get_function(&self, name: &'symbol str) -> Option<&TypeCheckerFunction<'symbol>> {
        self.functions.get(name)
    }

    pub fn split_method_call_name(
        &self,
        from: &'symbol str,
    ) -> Option<(&'symbol str, &'symbol str)> {
        let splitted: Vec<&str> = from.split(".").collect();

        if let Some(methods_name) = splitted.first() {
            if let Some(method_name) = splitted.get(1) {
                return Some((methods_name, method_name));
            }
        }

        None
    }

    pub fn get_specific_method_definition(
        &self,
        methods_name: &'symbol str,
        method_name: &'symbol str,
    ) -> Option<&TypeCheckerMethod<'symbol>> {
        if let Some(methods) = self.methods.get(methods_name) {
            if let Some(method) = methods.iter().find(|method| method.0 == method_name) {
                return Some(&method.1);
            }
        }

        None
    }

    pub fn begin_scope(&mut self) {
        self.locals.push(HashMap::with_capacity(255));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.locals.pop();

        self.scope -= 1;
    }
}
