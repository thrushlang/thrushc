use ahash::AHashMap as HashMap;

use crate::frontend::{
    lexer::span::Span,
    types::{
        ast::{Ast, metadata::fnparam::FunctionParameterMetadata},
        semantic::linter::types::{
            LinterAssemblerFunctionInfo, LinterAssemblerFunctions, LinterConstantInfo,
            LinterEnumFieldInfo, LinterEnums, LinterEnumsFieldsInfo, LinterFunctionInfo,
            LinterFunctionParameterInfo, LinterFunctionParameters, LinterFunctions,
            LinterGlobalConstants, LinterGlobalStatics, LinterLLIInfo, LinterLLIs,
            LinterLocalConstants, LinterLocalInfo, LinterLocalStatics, LinterLocals,
            LinterStaticInfo, LinterStructFieldsInfo, LinterStructs,
        },
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

    global_statics: LinterGlobalStatics<'linter>,
    local_statics: LinterLocalStatics<'linter>,

    global_constants: LinterGlobalConstants<'linter>,
    local_constants: LinterLocalConstants<'linter>,

    enums: LinterEnums<'linter>,
    structs: LinterStructs<'linter>,
    locals: LinterLocals<'linter>,
    llis: LinterLLIs<'linter>,
    parameters: LinterFunctionParameters<'linter>,
    scope: usize,
}

impl LinterSymbolsTable<'_> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::with_capacity(MINIMAL_FUNCTIONS_CAPACITY),
            asm_functions: HashMap::with_capacity(MINIMAL_ASM_FUNCTIONS_CAPACITY),

            global_statics: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            local_statics: Vec::with_capacity(MINIMAL_CONSTANTS_CAPACITY),

            global_constants: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            local_constants: Vec::with_capacity(MINIMAL_CONSTANTS_CAPACITY),

            enums: HashMap::with_capacity(MINIMAL_ENUMS_CAPACITY),
            structs: HashMap::with_capacity(MINIMAL_STRUCTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_LOCALS_CAPACITY),
            llis: Vec::with_capacity(MINIMAL_LLIS_CAPACITY),
            parameters: Vec::with_capacity(MINIMAL_PARAMETERS_CAPACITY),
            scope: 0,
        }
    }
}

impl<'linter> LinterSymbolsTable<'linter> {
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

    pub fn get_parameter_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterFunctionParameterInfo> {
        for scope in self.parameters.iter_mut().rev() {
            if let Some(parameter) = scope.get_mut(name) {
                return Some(parameter);
            }
        }

        None
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

    pub fn get_local_info(&mut self, name: &'linter str) -> Option<&mut LinterLocalInfo> {
        for scope in self.locals.iter_mut().rev() {
            if let Some(local) = scope.get_mut(name) {
                return Some(local);
            }
        }

        None
    }

    pub fn get_constant_info(&mut self, name: &'linter str) -> Option<&mut LinterConstantInfo> {
        for scope in self.local_constants.iter_mut().rev() {
            if let Some(local) = scope.get_mut(name) {
                return Some(local);
            }
        }

        if let Some(global) = self.global_constants.get_mut(name) {
            return Some(global);
        }

        None
    }

    pub fn get_static_info(&mut self, name: &'linter str) -> Option<&mut LinterStaticInfo> {
        for scope in self.local_statics.iter_mut().rev() {
            if let Some(staticvar) = scope.get_mut(name) {
                return Some(staticvar);
            }
        }

        if let Some(glstatic) = self.global_statics.get_mut(name) {
            return Some(glstatic);
        }

        None
    }

    pub fn get_lli_info(&mut self, name: &'linter str) -> Option<&mut LinterLLIInfo<'_>> {
        for scope in self.llis.iter_mut().rev() {
            if let Some(lli) = scope.get_mut(name) {
                return Some(lli);
            }
        }

        None
    }
}

impl<'linter> LinterSymbolsTable<'linter> {
    #[inline]
    pub fn get_all_function_parameters(&self) -> &LinterFunctionParameters<'_> {
        &self.parameters
    }

    #[inline]
    pub fn get_all_locals(&self) -> &LinterLocals<'_> {
        &self.locals
    }

    #[inline]
    pub fn get_all_llis(&self) -> &LinterLLIs<'_> {
        &self.llis
    }

    #[inline]
    pub fn get_all_enums(&self) -> &LinterEnums<'linter> {
        &self.enums
    }

    #[inline]
    pub fn get_all_global_constants(&self) -> &LinterGlobalConstants<'_> {
        &self.global_constants
    }

    #[inline]
    pub fn get_all_global_statics(&self) -> &LinterGlobalStatics<'_> {
        &self.global_statics
    }

    #[inline]
    pub fn get_all_local_constants(&self) -> &LinterLocalConstants<'_> {
        &self.local_constants
    }

    #[inline]
    pub fn get_all_locals_statics(&self) -> &LinterLocalStatics<'_> {
        &self.local_statics
    }

    #[inline]
    pub fn get_all_structs(&self) -> &LinterStructs<'_> {
        &self.structs
    }

    #[inline]
    pub fn get_all_functions(&self) -> &LinterFunctions<'_> {
        &self.functions
    }

    #[inline]
    pub fn get_all_asm_functions(&self) -> &LinterAssemblerFunctions<'_> {
        &self.asm_functions
    }
}

impl<'linter> LinterSymbolsTable<'linter> {
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

    pub fn new_global_constant(&mut self, name: &'linter str, info: LinterConstantInfo) {
        self.global_constants.insert(name, info);
    }

    pub fn new_local_constant(&mut self, name: &'linter str, info: LinterConstantInfo) {
        if let Some(scope) = self.local_constants.last_mut() {
            scope.insert(name, info);
        }
    }

    pub fn new_global_static(&mut self, name: &'linter str, info: LinterStaticInfo) {
        self.global_statics.insert(name, info);
    }

    pub fn new_local_static(&mut self, name: &'linter str, info: LinterStaticInfo) {
        if let Some(scope) = self.local_statics.last_mut() {
            scope.insert(name, info);
        }
    }

    pub fn new_parameter(&mut self, name: &'linter str, info: LinterFunctionParameterInfo) {
        if let Some(scope) = self.parameters.last_mut() {
            scope.insert(name, info);
        }
    }

    pub fn new_enum(&mut self, name: &'linter str, info: LinterEnumsFieldsInfo<'linter>) {
        self.enums.insert(name, info);
    }

    pub fn new_struct(&mut self, name: &'linter str, info: LinterStructFieldsInfo<'linter>) {
        self.structs.insert(name, info);
    }
}

impl LinterSymbolsTable<'_> {
    pub fn begin_scope(&mut self) {
        self.parameters.push(HashMap::with_capacity(255));
        self.local_constants.push(HashMap::with_capacity(255));
        self.locals.push(HashMap::with_capacity(255));
        self.llis.push(HashMap::with_capacity(255));

        self.scope += 1;
    }

    pub fn end_scope(&mut self) {
        self.parameters.pop();
        self.local_constants.pop();
        self.locals.pop();
        self.llis.pop();

        self.scope -= 1;
    }
}

impl<'linter> LinterSymbolsTable<'linter> {
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

    pub fn bulk_declare_parameters(&mut self, parameters: &'linter [Ast]) {
        parameters.iter().for_each(|parameter| {
            if let Ast::FunctionParameter {
                name,
                span,
                metadata,
                ..
            } = parameter
            {
                let metadata: &FunctionParameterMetadata = metadata;

                self.new_parameter(name, (*span, false, !metadata.is_mutable()));
            }
        });
    }
}
