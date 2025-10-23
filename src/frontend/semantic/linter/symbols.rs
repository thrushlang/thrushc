use ahash::AHashMap as HashMap;

use crate::frontend::lexer::span::Span;

use crate::frontend::semantic::linter::constants::LINTER_MINIMAL_GLOBAL_CAPACITY;
use crate::frontend::semantic::linter::constants::LINTER_MINIMAL_LOCAL_CAPACITY;

use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::frontend::types::parser::repr::FunctionParameter;

use crate::frontend::types::semantic::linter::types::LinterAssemblerFunctionInfo;
use crate::frontend::types::semantic::linter::types::LinterAssemblerFunctions;
use crate::frontend::types::semantic::linter::types::LinterConstantInfo;
use crate::frontend::types::semantic::linter::types::LinterEnumFieldInfo;
use crate::frontend::types::semantic::linter::types::LinterEnums;
use crate::frontend::types::semantic::linter::types::LinterEnumsFieldsInfo;
use crate::frontend::types::semantic::linter::types::LinterFunctionInfo;
use crate::frontend::types::semantic::linter::types::LinterFunctionParameterInfo;
use crate::frontend::types::semantic::linter::types::LinterFunctionParameters;
use crate::frontend::types::semantic::linter::types::LinterFunctions;
use crate::frontend::types::semantic::linter::types::LinterGlobalConstants;
use crate::frontend::types::semantic::linter::types::LinterGlobalStatics;
use crate::frontend::types::semantic::linter::types::LinterLLIInfo;
use crate::frontend::types::semantic::linter::types::LinterLLIs;
use crate::frontend::types::semantic::linter::types::LinterLocalConstants;
use crate::frontend::types::semantic::linter::types::LinterLocalInfo;
use crate::frontend::types::semantic::linter::types::LinterLocalStatics;
use crate::frontend::types::semantic::linter::types::LinterLocals;
use crate::frontend::types::semantic::linter::types::LinterStaticInfo;
use crate::frontend::types::semantic::linter::types::LinterStructFieldsInfo;
use crate::frontend::types::semantic::linter::types::LinterStructs;

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
            functions: HashMap::with_capacity(LINTER_MINIMAL_GLOBAL_CAPACITY),
            asm_functions: HashMap::with_capacity(LINTER_MINIMAL_GLOBAL_CAPACITY),

            global_statics: HashMap::with_capacity(LINTER_MINIMAL_GLOBAL_CAPACITY),
            local_statics: Vec::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY),

            global_constants: HashMap::with_capacity(LINTER_MINIMAL_GLOBAL_CAPACITY),
            local_constants: Vec::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY),

            enums: HashMap::with_capacity(LINTER_MINIMAL_GLOBAL_CAPACITY),
            structs: HashMap::with_capacity(LINTER_MINIMAL_GLOBAL_CAPACITY),
            locals: Vec::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY),
            llis: Vec::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY),
            parameters: HashMap::with_capacity(15),
            scope: 0,
        }
    }
}

impl<'linter> LinterSymbolsTable<'linter> {
    #[must_use]
    pub fn get_asm_function_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterAssemblerFunctionInfo<'linter>> {
        self.asm_functions.get_mut(name)
    }

    #[must_use]
    pub fn get_function_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterFunctionInfo<'linter>> {
        self.functions.get_mut(name)
    }

    #[must_use]
    pub fn get_parameter_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterFunctionParameterInfo> {
        self.parameters.get_mut(name)
    }

    #[must_use]
    pub fn get_enum_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterEnumsFieldsInfo<'linter>> {
        self.enums.get_mut(name)
    }

    #[must_use]
    pub fn get_struct_info(
        &mut self,
        name: &'linter str,
    ) -> Option<&mut LinterStructFieldsInfo<'linter>> {
        self.structs.get_mut(name)
    }

    #[must_use]
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

    #[must_use]
    pub fn get_local_info(&mut self, name: &'linter str) -> Option<&mut LinterLocalInfo> {
        for scope in self.locals.iter_mut().rev() {
            if let Some(local) = scope.get_mut(name) {
                return Some(local);
            }
        }

        None
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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
    #[inline]
    pub fn new_local(&mut self, name: &'linter str, info: LinterLocalInfo) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, info);
        }
    }

    #[inline]
    pub fn new_lli(&mut self, name: &'linter str, info: LinterLLIInfo) {
        if let Some(scope) = self.llis.last_mut() {
            scope.insert(name, info);
        }
    }

    #[inline]
    pub fn new_asm_function(
        &mut self,
        name: &'linter str,
        info: LinterAssemblerFunctionInfo<'linter>,
    ) {
        self.asm_functions.insert(name, info);
    }

    #[inline]
    pub fn new_function(&mut self, name: &'linter str, info: LinterFunctionInfo<'linter>) {
        self.functions.insert(name, info);
    }

    #[inline]
    pub fn new_global_constant(&mut self, name: &'linter str, info: LinterConstantInfo) {
        self.global_constants.insert(name, info);
    }

    #[inline]
    pub fn new_local_constant(&mut self, name: &'linter str, info: LinterConstantInfo) {
        if let Some(scope) = self.local_constants.last_mut() {
            scope.insert(name, info);
        }
    }

    #[inline]
    pub fn new_global_static(&mut self, name: &'linter str, info: LinterStaticInfo) {
        self.global_statics.insert(name, info);
    }

    #[inline]
    pub fn new_local_static(&mut self, name: &'linter str, info: LinterStaticInfo) {
        if let Some(scope) = self.local_statics.last_mut() {
            scope.insert(name, info);
        }
    }

    #[inline]
    pub fn new_parameter(&mut self, name: &'linter str, info: LinterFunctionParameterInfo) {
        self.parameters.insert(name, info);
    }

    #[inline]
    pub fn new_enum(&mut self, name: &'linter str, info: LinterEnumsFieldsInfo<'linter>) {
        self.enums.insert(name, info);
    }

    #[inline]
    pub fn new_struct(&mut self, name: &'linter str, info: LinterStructFieldsInfo<'linter>) {
        self.structs.insert(name, info);
    }
}

impl LinterSymbolsTable<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.local_constants
            .push(HashMap::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY));
        self.locals
            .push(HashMap::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY));
        self.llis
            .push(HashMap::with_capacity(LINTER_MINIMAL_LOCAL_CAPACITY));

        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.local_constants.pop();
        self.locals.pop();
        self.llis.pop();

        self.scope -= 1;
    }
}

impl<'linter> LinterSymbolsTable<'linter> {
    #[inline]
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
}

impl<'linter> LinterSymbolsTable<'linter> {
    #[inline]
    pub fn declare_parameters(&mut self, parameters: &'linter [Ast]) {
        parameters.iter().for_each(|parameter| {
            let parameter: FunctionParameter = parameter.as_function_parameter();

            let name: &str = parameter.0;
            let metadata: FunctionParameterMetadata = parameter.5;
            let span: Span = parameter.4;

            self.new_parameter(name, (span, false, !metadata.is_mutable()));
        });
    }

    #[inline]
    pub fn finish_parameters(&mut self) {
        self.parameters.clear();
    }
}
