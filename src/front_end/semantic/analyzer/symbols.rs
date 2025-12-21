use crate::front_end::types::semantic::analyzer::types::AnalyzerAssemblerFunction;
use crate::front_end::types::semantic::analyzer::types::AnalyzerAssemblerFunctions;
use crate::front_end::types::semantic::analyzer::types::AnalyzerFunction;
use crate::front_end::types::semantic::analyzer::types::AnalyzerFunctions;
use crate::front_end::types::semantic::analyzer::types::AnalyzerLLI;
use crate::front_end::types::semantic::analyzer::types::AnalyzerLLIs;
use crate::front_end::types::semantic::analyzer::types::AnalyzerLocal;
use crate::front_end::types::semantic::analyzer::types::AnalyzerLocals;

use ahash::AHashMap as HashMap;

pub const PREALLOCATE_SYMBOLS_GLOBAL_TABLE: usize = 1000;
pub const PREALLOCATE_SYMBOLS_LOCAL_TABLE: usize = 255;

#[derive(Debug)]
pub struct AnalyzerSymbolsTable<'symbol> {
    functions: AnalyzerFunctions<'symbol>,
    asm_functions: AnalyzerAssemblerFunctions<'symbol>,

    locals: AnalyzerLocals<'symbol>,
    llis: AnalyzerLLIs<'symbol>,

    scope: usize,
}

impl<'symbol> AnalyzerSymbolsTable<'symbol> {
    #[inline]
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(PREALLOCATE_SYMBOLS_GLOBAL_TABLE),
            asm_functions: HashMap::with_capacity(PREALLOCATE_SYMBOLS_GLOBAL_TABLE),

            locals: Vec::with_capacity(PREALLOCATE_SYMBOLS_LOCAL_TABLE),
            llis: Vec::with_capacity(PREALLOCATE_SYMBOLS_LOCAL_TABLE),

            scope: 0,
        }
    }
}

impl<'symbol> AnalyzerSymbolsTable<'symbol> {
    #[inline]
    pub fn new_local(&mut self, name: &'symbol str, local: AnalyzerLocal<'symbol>) {
        if let Some(last_scope) = self.locals.last_mut() {
            last_scope.insert(name, local);
        }
    }

    #[inline]
    pub fn new_lli(&mut self, name: &'symbol str, lli: AnalyzerLLI<'symbol>) {
        if let Some(last_scope) = self.llis.last_mut() {
            last_scope.insert(name, lli);
        }
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
        self.llis
            .push(HashMap::with_capacity(PREALLOCATE_SYMBOLS_LOCAL_TABLE));
        self.locals
            .push(HashMap::with_capacity(PREALLOCATE_SYMBOLS_LOCAL_TABLE));

        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.llis.pop();
        self.locals.pop();

        self.scope -= 1;
    }
}
