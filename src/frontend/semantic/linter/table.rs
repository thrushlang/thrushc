use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::span::Span,
    types::frontend::{
        linter::types::{
            LinterConstantInfo, LinterConstants, LinterFunctionInfo, LinterFunctionParameterInfo,
            LinterFunctionParameters, LinterFunctions, LinterLLIInfo, LinterLLIs, LinterLocalInfo,
            LinterLocals,
        },
        parser::stmts::stmt::ThrushStatement,
    },
};

const MINIMAL_FUNCTIONS_CAPACITY: usize = 255;
const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_LOCALS_CAPACITY: usize = 255;
const MINIMAL_LLIS_CAPACITY: usize = 255;
const MINIMAL_PARAMETERS_CAPACITY: usize = 10;

pub struct LinterSymbolsTable<'linter> {
    functions: LinterFunctions<'linter>,
    constants: LinterConstants<'linter>,
    locals: LinterLocals<'linter>,
    llis: LinterLLIs<'linter>,
    parameters: LinterFunctionParameters<'linter>,
    scope: usize,
}

impl<'linter> LinterSymbolsTable<'linter> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(MINIMAL_FUNCTIONS_CAPACITY),
            constants: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_LOCALS_CAPACITY),
            llis: Vec::with_capacity(MINIMAL_LLIS_CAPACITY),
            parameters: HashMap::with_capacity(MINIMAL_PARAMETERS_CAPACITY),
            scope: 0,
        }
    }

    pub fn new_function(&mut self, name: &'linter str, info: LinterFunctionInfo<'linter>) {
        self.functions.insert(name, info);
    }

    pub fn new_constant(&mut self, name: &'linter str, info: LinterConstantInfo) {
        self.constants.insert(name, info);
    }

    pub fn new_parameter(&mut self, name: &'linter str, info: LinterFunctionParameterInfo) {
        self.parameters.insert(name, info);
    }

    pub fn get_all_parameters(&self) -> &HashMap<&'linter str, (Span, bool, bool)> {
        &self.parameters
    }

    pub fn get_all_locals(&self) -> &[HashMap<&'linter str, (Span, bool, bool)>] {
        &self.locals
    }

    pub fn get_all_llis(&self) -> &[HashMap<&'linter str, (Span, bool)>] {
        &self.llis
    }

    pub fn get_all_constants(&self) -> &HashMap<&'linter str, (Span, bool)> {
        &self.constants
    }

    pub fn get_all_functions(&self) -> &HashMap<&'linter str, (Span, bool)> {
        &self.functions
    }

    pub fn new_local(&mut self, name: &'linter str, info: LinterLocalInfo) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, info);
        }
    }

    pub fn new_lli(&mut self, name: &'linter str, info: LinterLLIInfo) {
        if let Some(scope) = self.llis.last_mut() {
            scope.insert(name, info);
        }
    }

    pub fn bulk_declare_parameters(&mut self, parameters: &'linter [ThrushStatement]) {
        parameters.iter().for_each(|parameter| {
            if let ThrushStatement::FunctionParameter {
                name,
                is_mutable,
                span,
                ..
            } = parameter
            {
                self.new_parameter(name, (*span, false, !is_mutable));
            }
        });
    }

    pub fn destroy_all_parameters(&mut self) {
        self.parameters.clear();
    }

    pub fn get_function_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterFunctionInfo<'linter>> {
        self.functions.get_mut(name)
    }

    pub fn get_constant_info(&mut self, name: &'linter str) -> Option<&mut LinterConstantInfo> {
        self.constants.get_mut(name)
    }

    pub fn get_parameter_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterFunctionParameterInfo> {
        self.parameters.get_mut(name)
    }

    pub fn get_local_info(&mut self, name: &'linter str) -> Option<&mut LinterLocalInfo> {
        for i in (0..=self.scope - 1).rev() {
            if self.locals[i].contains_key(name) {
                return Some(self.locals[i].get_mut(name).unwrap());
            }
        }

        None
    }

    pub fn get_lli_info(&mut self, name: &'linter str) -> Option<&mut LinterLLIInfo> {
        for i in (0..=self.scope - 1).rev() {
            if self.llis[i].contains_key(name) {
                return Some(self.llis[i].get_mut(name).unwrap());
            }
        }

        None
    }

    pub fn begin_scope(&mut self) {
        self.locals.push(HashMap::with_capacity(255));
        self.llis.push(HashMap::with_capacity(255));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.locals.pop();
        self.llis.pop();

        self.scope -= 1;
    }
}
