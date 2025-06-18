use ahash::AHashMap as HashMap;

use crate::frontend::{
    lexer::span::Span,
    types::parser::stmts::stmt::ThrushStatement,
    types::semantic::linter::types::{
        LinterAssemblerFunctionInfo, LinterAssemblerFunctions, LinterConstantInfo, LinterConstants,
        LinterEnumFieldInfo, LinterEnums, LinterEnumsFieldsInfo, LinterFunctionInfo,
        LinterFunctionParameterInfo, LinterFunctionParameters, LinterFunctions, LinterLLIInfo,
        LinterLLIs, LinterLocalInfo, LinterLocals, LinterStructFieldInfo, LinterStructFieldsInfo,
        LinterStructs,
    },
};

const MINIMAL_FUNCTIONS_CAPACITY: usize = 255;
const MINIMAL_ASM_FUNCTIONS_CAPACITY: usize = 255;
const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_LOCALS_CAPACITY: usize = 255;
const MINIMAL_LLIS_CAPACITY: usize = 255;
const MINIMAL_ENUMS_CAPACITY: usize = 255;
const MINIMAL_STRUCTS_CAPACITY: usize = 255;
const MINIMAL_PARAMETERS_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct LinterSymbolsTable<'linter> {
    functions: LinterFunctions<'linter>,
    asm_functions: LinterAssemblerFunctions<'linter>,
    constants: LinterConstants<'linter>,
    enums: LinterEnums<'linter>,
    structs: LinterStructs<'linter>,
    locals: LinterLocals<'linter>,
    llis: LinterLLIs<'linter>,
    parameters: LinterFunctionParameters<'linter>,
    scope: usize,
}

impl<'linter> LinterSymbolsTable<'linter> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(MINIMAL_FUNCTIONS_CAPACITY),
            asm_functions: HashMap::with_capacity(MINIMAL_ASM_FUNCTIONS_CAPACITY),
            constants: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            enums: HashMap::with_capacity(MINIMAL_ENUMS_CAPACITY),
            structs: HashMap::with_capacity(MINIMAL_STRUCTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_LOCALS_CAPACITY),
            llis: Vec::with_capacity(MINIMAL_LLIS_CAPACITY),
            parameters: HashMap::with_capacity(MINIMAL_PARAMETERS_CAPACITY),
            scope: 0,
        }
    }

    pub fn new_asm_function(
        &mut self,
        name: &'linter str,
        info: LinterAssemblerFunctionInfo<'linter>,
    ) {
        self.asm_functions.insert(name, info);
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

    pub fn new_enum(&mut self, name: &'linter str, info: LinterEnumsFieldsInfo<'linter>) {
        self.enums.insert(name, info);
    }

    pub fn new_struct(&mut self, name: &'linter str, info: LinterStructFieldsInfo<'linter>) {
        self.structs.insert(name, info);
    }

    pub fn get_all_function_parameters(&self) -> &LinterFunctionParameters {
        &self.parameters
    }

    pub fn get_all_locals(&self) -> &LinterLocals {
        &self.locals
    }

    pub fn get_all_llis(&self) -> &LinterLLIs {
        &self.llis
    }

    pub fn get_all_enums(&self) -> &LinterEnums<'linter> {
        &self.enums
    }

    pub fn get_all_constants(&self) -> &LinterConstants {
        &self.constants
    }

    pub fn get_all_structs(&self) -> &LinterStructs {
        &self.structs
    }

    pub fn get_all_functions(&self) -> &LinterFunctions {
        &self.functions
    }

    pub fn get_all_asm_functions(&self) -> &LinterAssemblerFunctions {
        &self.asm_functions
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

    pub fn get_asm_function_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterAssemblerFunctionInfo<'linter>> {
        self.asm_functions.get_mut(name)
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

    pub fn get_enum_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterEnumsFieldsInfo<'linter>> {
        self.enums.get_mut(name)
    }

    pub fn get_struct_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterStructFieldsInfo<'linter>> {
        self.structs.get_mut(name)
    }

    pub fn split_property_name(
        &mut self,
        from: &'linter str,
    ) -> Option<(&'linter str, &'linter str)> {
        let splitted: Vec<&str> = from.split(".").collect();

        if let Some(struct_name) = splitted.first() {
            if let Some(field_name) = splitted.get(1) {
                return Some((struct_name, field_name));
            }
        }

        None
    }

    pub fn get_enum_field_info(
        &mut self,
        enum_name: &'linter str,
        field_name: &'linter str,
    ) -> Option<&mut LinterEnumFieldInfo> {
        if let Some(raw_enum_fields) = self.get_enum_info(enum_name) {
            let enum_fields: &mut HashMap<&'linter str, (Span, bool)> = &mut raw_enum_fields.0;

            if let Some(enum_field) = enum_fields.get_mut(field_name) {
                return Some(enum_field);
            }
        }

        None
    }

    pub fn split_enum_field_name(
        &mut self,
        from: &'linter str,
    ) -> Option<(&'linter str, &'linter str)> {
        let splitted: Vec<&str> = from.split(".").collect();

        if let Some(enum_name) = splitted.first() {
            if let Some(field_name) = splitted.get(1) {
                return Some((enum_name, field_name));
            }
        }

        None
    }

    pub fn get_local_info(&mut self, name: &'linter str) -> Option<&mut LinterLocalInfo> {
        for scope in self.locals.iter_mut().rev() {
            if let Some(local) = scope.get_mut(name) {
                return Some(local);
            }
        }

        None
    }

    pub fn get_lli_info(&mut self, name: &'linter str) -> Option<&mut LinterLLIInfo> {
        for scope in self.llis.iter_mut().rev() {
            if let Some(lli) = scope.get_mut(name) {
                return Some(lli);
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
