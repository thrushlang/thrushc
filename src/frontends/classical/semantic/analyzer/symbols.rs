use ahash::AHashMap as HashMap;

use crate::frontends::classical::types::semantic::analyzer::types::{
    AnalyzerAssemblerFunction, AnalyzerAssemblerFunctions, AnalyzerFunction, AnalyzerFunctions,
    AnalyzerLLI, AnalyzerLLIs, AnalyzerLocal, AnalyzerLocals,
};

#[derive(Debug)]
pub struct AnalyzerSymbolsTable<'symbol> {
    functions: AnalyzerFunctions<'symbol>,
    asm_functions: AnalyzerAssemblerFunctions<'symbol>,

    locals: AnalyzerLocals<'symbol>,
    llis: AnalyzerLLIs<'symbol>,

    scope: usize,
}

impl<'symbol> AnalyzerSymbolsTable<'symbol> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(100),
            asm_functions: HashMap::with_capacity(100),

            locals: Vec::with_capacity(255),
            llis: Vec::with_capacity(255),

            scope: 0,
        }
    }
}

impl<'symbol> AnalyzerSymbolsTable<'symbol> {
    #[inline]
    pub fn new_local(&mut self, name: &'symbol str, local: AnalyzerLocal<'symbol>) {
        self.locals.last_mut().unwrap().insert(name, local);
    }

    #[inline]
    pub fn new_lli(&mut self, name: &'symbol str, lli: AnalyzerLLI<'symbol>) {
        self.llis.last_mut().unwrap().insert(name, lli);
    }

    #[inline]
    pub fn new_asm_function(
        &mut self,
        name: &'symbol str,
        function: AnalyzerAssemblerFunction<'symbol>,
    ) {
        self.asm_functions.insert(name, function);
    }

    #[inline]
    pub fn new_function(&mut self, name: &'symbol str, function: AnalyzerFunction<'symbol>) {
        self.functions.insert(name, function);
    }
}

impl AnalyzerSymbolsTable<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.llis.push(HashMap::with_capacity(255));
        self.locals.push(HashMap::with_capacity(255));

        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.llis.pop();
        self.locals.pop();

        self.scope -= 1;
    }
}
