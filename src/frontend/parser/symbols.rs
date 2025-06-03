use crate::{
    frontend::lexer::span::Span,
    standard::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    types::frontend::{
        lexer::types::MethodsApplicant,
        parser::symbols::types::{
            ConstantSymbol, Constants, CustomTypeSymbol, CustomTypes, EnumSymbol, Enums,
            FoundSymbolId, Function, Functions, LLISymbol, LLIs, LocalSymbol, Locals, Methods,
            ParameterSymbol, Parameters, Struct, Structs,
        },
    },
};

use ahash::AHashMap as HashMap;

const MINIMAL_CUSTOM_TYPE_CAPACITY: usize = 255;
const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_STRUCTURE_CAPACITY: usize = 255;
const MINIMAL_ENUMS_CAPACITY: usize = 255;
const MINIMAL_LOCALS_CAPACITY: usize = 255;
const MINIMAL_LLIS_CAPACITY: usize = 255;
const MINIMAL_PARAMETERS_CAPACITY: usize = 255;

#[derive(Clone, Debug, Default)]
pub struct SymbolsTable<'instr> {
    custom_types: CustomTypes<'instr>,
    constants: Constants<'instr>,
    locals: Locals<'instr>,
    llis: LLIs<'instr>,
    structs: Structs<'instr>,
    functions: Functions<'instr>,
    enums: Enums<'instr>,
    parameters: Parameters<'instr>,
}

impl<'instr> SymbolsTable<'instr> {
    pub fn with_functions(functions: HashMap<&'instr str, Function<'instr>>) -> Self {
        Self {
            custom_types: HashMap::with_capacity(MINIMAL_CUSTOM_TYPE_CAPACITY),
            constants: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_LOCALS_CAPACITY),
            llis: Vec::with_capacity(MINIMAL_LLIS_CAPACITY),
            functions,
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
            enums: HashMap::with_capacity(MINIMAL_ENUMS_CAPACITY),
            parameters: HashMap::with_capacity(MINIMAL_PARAMETERS_CAPACITY),
        }
    }

    pub fn begin_scope(&mut self) {
        self.locals
            .push(HashMap::with_capacity(MINIMAL_LOCALS_CAPACITY));

        self.llis
            .push(HashMap::with_capacity(MINIMAL_LLIS_CAPACITY));
    }

    pub fn end_scope(&mut self) {
        self.locals.pop();
        self.llis.pop();
    }

    pub fn end_parameters(&mut self) {
        self.parameters.clear();
    }

    pub fn clear_all_scopes(&mut self) {
        self.locals.clear();
    }
}

impl<'instr> SymbolsTable<'instr> {
    pub fn new_parameter(
        &mut self,
        name: &'instr str,
        parameter: ParameterSymbol<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.parameters.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Parameter already declared"),
                format!("'{}' parameter already declared before.", name),
                None,
                span,
            ));
        }

        self.parameters.insert(name, parameter);

        Ok(())
    }

    pub fn new_lli(
        &mut self,
        name: &'instr str,
        lli: LLISymbol<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let Some(last_scope) = self.llis.last_mut() {
            if last_scope.contains_key(name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Low level instruction already declared"),
                    format!("Low level instruction '{}' already declared before.", name),
                    None,
                    span,
                ));
            }

            last_scope.insert(name, lli);

            return Ok(());
        }

        return Err(ThrushCompilerIssue::Bug(
            String::from("Low level instruction not caught"),
            String::from("The final scope was not obtained."),
            span,
            CompilationPosition::Parser,
            line!(),
        ));
    }

    pub fn new_local(
        &mut self,
        name: &'instr str,
        local: LocalSymbol<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let Some(last_scope) = self.locals.last_mut() {
            if last_scope.contains_key(name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Local variable already declared"),
                    format!("'{}' local variable already declared before.", name),
                    None,
                    span,
                ));
            }

            last_scope.insert(name, local);

            return Ok(());
        }

        return Err(ThrushCompilerIssue::Bug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            line!(),
        ));
    }

    pub fn new_constant(
        &mut self,
        name: &'instr str,
        constant: ConstantSymbol<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Constant already declared"),
                format!("'{}' constant already declared before.", name),
                None,
                span,
            ));
        }

        self.constants.insert(name, constant);

        Ok(())
    }

    pub fn new_custom_type(
        &mut self,
        name: &'instr str,
        custom_type: CustomTypeSymbol<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Custom type already declared"),
                format!("'{}' custom type already declared before.", name),
                None,
                span,
            ));
        }

        self.custom_types.insert(name, custom_type);

        Ok(())
    }

    pub fn new_struct(
        &mut self,
        name: &'instr str,
        field_types: Struct<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.structs.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Structure already declared"),
                format!("'{}' structure already declared before.", name),
                None,
                span,
            ));
        }

        self.structs.insert(name, field_types);

        Ok(())
    }

    pub fn new_enum(
        &mut self,
        name: &'instr str,
        union: EnumSymbol<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.enums.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Enum already declared"),
                format!("'{}' enum already declared before.", name),
                None,
                span,
            ));
        }

        self.enums.insert(name, union);

        Ok(())
    }

    pub fn new_function(
        &mut self,
        name: &'instr str,
        function: Function<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.functions.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Function already declared"),
                format!("'{}' function already declared before.", name),
                None,
                span,
            ));
        }

        self.functions.insert(name, function);

        Ok(())
    }

    pub fn add_methods(
        &mut self,
        name: &str,
        methods: Methods<'instr>,
        applicant: MethodsApplicant,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        match applicant {
            MethodsApplicant::Struct => {
                let structure: &mut Struct = self.get_struct_mut(name, span)?;
                structure.3 = methods;
            }
        }

        Ok(())
    }

    pub fn contains_structure(&self, name: &str, span: Span) -> Result<(), ThrushCompilerIssue> {
        if !self.structs.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Structure not found"),
                format!("'{}' structure not defined or declared yet.", name),
                None,
                span,
            ));
        }

        Ok(())
    }
}

impl<'instr> SymbolsTable<'instr> {
    pub fn get_symbols_id(
        &self,
        name: &'instr str,
        span: Span,
    ) -> Result<FoundSymbolId<'instr>, ThrushCompilerIssue> {
        if self.custom_types.contains_key(name) {
            return Ok((None, None, None, None, Some(name), None, None, None));
        }

        if self.constants.contains_key(name) {
            return Ok((None, None, None, Some(name), None, None, None, None));
        }

        if self.structs.contains_key(name) {
            return Ok((Some(name), None, None, None, None, None, None, None));
        }

        if self.enums.contains_key(name) {
            return Ok((None, None, Some(name), None, None, None, None, None));
        }

        if self.functions.contains_key(name) {
            return Ok((None, Some(name), None, None, None, None, None, None));
        }

        if self.parameters.contains_key(name) {
            return Ok((None, None, None, None, None, Some(name), None, None));
        }

        for (idx, scope) in self.llis.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((None, None, None, None, None, None, Some((name, idx)), None));
            }
        }

        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((None, None, None, None, None, None, None, Some((name, idx))));
            }
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            format!("'{}' isn't declared or defined.", name),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_lli_by_id(
        &self,
        lli_id: &'instr str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&LLISymbol<'instr>, ThrushCompilerIssue> {
        if let Some(lli) = self.llis[scope_idx].get(lli_id) {
            return Ok(lli);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("LLI not found at local scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct_by_id(
        &self,
        struct_id: &'instr str,
        span: Span,
    ) -> Result<Struct<'instr>, ThrushCompilerIssue> {
        if let Some(structure) = self.structs.get(struct_id).cloned() {
            return Ok(structure);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Struct not found at global scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_function_by_id(
        &self,
        span: Span,
        func_id: &'instr str,
    ) -> Result<Function<'instr>, ThrushCompilerIssue> {
        if let Some(function) = self.functions.get(func_id).cloned() {
            return Ok(function);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Function not found at global scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_enum_by_id(
        &self,
        enum_id: &'instr str,
        span: Span,
    ) -> Result<EnumSymbol<'instr>, ThrushCompilerIssue> {
        if let Some(enum_found) = self.enums.get(enum_id).cloned() {
            return Ok(enum_found);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Enum not found at global scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_custom_type_by_id(
        &self,
        custom_type_id: &'instr str,
        span: Span,
    ) -> Result<CustomTypeSymbol<'instr>, ThrushCompilerIssue> {
        if let Some(custom_type) = self.custom_types.get(custom_type_id).cloned() {
            return Ok(custom_type);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Custom type not found at global scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_local_by_id(
        &self,
        local_id: &'instr str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&LocalSymbol<'instr>, ThrushCompilerIssue> {
        if let Some(local) = self.locals[scope_idx].get(local_id) {
            return Ok(local);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Local not found at local scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_const_by_id(
        &self,
        const_id: &'instr str,
        span: Span,
    ) -> Result<ConstantSymbol<'instr>, ThrushCompilerIssue> {
        if let Some(constant) = self.constants.get(const_id).cloned() {
            return Ok(constant);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Constant not found at global scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_parameter_by_id(
        &self,
        parameter_id: &'instr str,
        span: Span,
    ) -> Result<ParameterSymbol<'instr>, ThrushCompilerIssue> {
        if let Some(parameter) = self.parameters.get(parameter_id).cloned() {
            return Ok(parameter);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Parameter not found in this scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct(
        &self,
        name: &str,
        span: Span,
    ) -> Result<Struct<'instr>, ThrushCompilerIssue> {
        if let Some(struct_fields) = self.structs.get(name).cloned() {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct_mut(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<&mut Struct<'instr>, ThrushCompilerIssue> {
        if let Some(struct_fields) = self.structs.get_mut(name) {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            None,
            span,
        ))
    }
}
