use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        types::{
            ast::Ast,
            parser::symbols::types::{
                AssemblerFunction, AssemblerFunctions, ConstantSymbol, CustomTypeSymbol,
                CustomTypes, EnumSymbol, Enums, FoundSymbolId, Function, Functions,
                GlobalConstants, GlobalStatics, LLISymbol, LLIs, LocalConstants, LocalStatics,
                LocalSymbol, Locals, ParameterSymbol, Parameters, StaticSymbol, Struct, Structs,
            },
        },
    },
};

use ahash::AHashMap as HashMap;

#[derive(Clone, Debug, Default)]
pub struct SymbolsTable<'parser> {
    custom_types: CustomTypes<'parser>,

    global_statics: GlobalStatics<'parser>,
    statics: LocalStatics<'parser>,

    global_constants: GlobalConstants<'parser>,
    constants: LocalConstants<'parser>,

    locals: Locals<'parser>,
    llis: LLIs<'parser>,
    structs: Structs<'parser>,
    functions: Functions<'parser>,
    asm_functions: AssemblerFunctions<'parser>,
    enums: Enums<'parser>,
    parameters: Parameters<'parser>,
}

impl<'parser> SymbolsTable<'parser> {
    pub fn with_functions(
        functions: Functions<'parser>,
        asm_functions: AssemblerFunctions<'parser>,
    ) -> Self {
        Self {
            custom_types: HashMap::with_capacity(255),

            global_statics: HashMap::with_capacity(255),
            statics: Vec::with_capacity(255),

            global_constants: HashMap::with_capacity(255),
            constants: Vec::with_capacity(255),

            locals: Vec::with_capacity(255),
            llis: Vec::with_capacity(255),

            functions,
            asm_functions,

            structs: HashMap::with_capacity(255),
            enums: HashMap::with_capacity(255),
            parameters: HashMap::with_capacity(255),
        }
    }
}

impl SymbolsTable<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.statics.push(HashMap::with_capacity(255));
        self.constants.push(HashMap::with_capacity(255));
        self.locals.push(HashMap::with_capacity(255));
        self.llis.push(HashMap::with_capacity(255));
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.statics.pop();
        self.constants.pop();
        self.locals.pop();
        self.llis.pop();
    }

    #[inline]
    pub fn end_parameters(&mut self) {
        self.parameters.clear();
    }
}

impl<'parser> SymbolsTable<'parser> {
    pub fn start_parameters(
        &mut self,
        parameters: &[Ast<'parser>],
    ) -> Result<(), ThrushCompilerIssue> {
        for parameter in parameters.iter() {
            if let Ast::FunctionParameter {
                name,
                kind,
                span,
                metadata,
                ..
            } = parameter
            {
                if self.parameters.contains_key(name) {
                    return Err(ThrushCompilerIssue::Error(
                        "Parameter already declared".into(),
                        format!("'{}' parameter already declared before.", name),
                        None,
                        *span,
                    ));
                }

                self.parameters
                    .insert(name, (kind.clone(), *metadata, *span));
            }
        }

        Ok(())
    }

    pub fn new_lli(
        &mut self,
        name: &'parser str,
        lli: LLISymbol<'parser>,
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

        return Err(ThrushCompilerIssue::FrontEndBug(
            String::from("Low level instruction not caught"),
            String::from("The final scope was not obtained."),
            span,
            CompilationPosition::Parser,
            line!(),
        ));
    }

    pub fn new_local(
        &mut self,
        name: &'parser str,
        local: LocalSymbol<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let Some(last_scope) = self.locals.last_mut() {
            if last_scope.contains_key(name) || self.parameters.contains_key(name) {
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

        return Err(ThrushCompilerIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            line!(),
        ));
    }

    pub fn new_global_static(
        &mut self,
        name: &'parser str,
        static_: StaticSymbol<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.global_statics.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                "Static already declared".into(),
                format!("'{}' static already declared before.", name),
                None,
                span,
            ));
        }

        self.global_statics.insert(name, static_);

        Ok(())
    }

    pub fn new_static(
        &mut self,
        name: &'parser str,
        static_: StaticSymbol<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let Some(last_scope) = self.statics.last_mut() {
            if last_scope.contains_key(name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Static already declared"),
                    format!("'{}' static already declared before.", name),
                    None,
                    span,
                ));
            }

            last_scope.insert(name, static_);

            return Ok(());
        }

        return Err(ThrushCompilerIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            line!(),
        ));
    }

    pub fn new_global_constant(
        &mut self,
        name: &'parser str,
        constant: ConstantSymbol<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.global_constants.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                "Constant already declared".into(),
                format!("'{}' constant already declared before.", name),
                None,
                span,
            ));
        }

        self.global_constants.insert(name, constant);

        Ok(())
    }

    pub fn new_constant(
        &mut self,
        name: &'parser str,
        constant: ConstantSymbol<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if let Some(last_scope) = self.constants.last_mut() {
            if last_scope.contains_key(name) {
                return Err(ThrushCompilerIssue::Error(
                    String::from("Constant already declared"),
                    format!("'{}' constant already declared before.", name),
                    None,
                    span,
                ));
            }

            last_scope.insert(name, constant);

            return Ok(());
        }

        return Err(ThrushCompilerIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            line!(),
        ));
    }

    pub fn new_custom_type(
        &mut self,
        name: &'parser str,
        custom_type: CustomTypeSymbol<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.custom_types.contains_key(name) {
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
        name: &'parser str,
        field_types: Struct<'parser>,
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
        name: &'parser str,
        union: EnumSymbol<'parser>,
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

    pub fn new_asm_function(
        &mut self,
        name: &'parser str,
        function: AssemblerFunction<'parser>,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
        if self.asm_functions.contains_key(name) {
            return Err(ThrushCompilerIssue::Error(
                String::from("Assembly function already declared"),
                format!("'{}' assembler function already declared before.", name),
                None,
                span,
            ));
        }

        self.asm_functions.insert(name, function);

        Ok(())
    }

    pub fn new_function(
        &mut self,
        name: &'parser str,
        function: Function<'parser>,
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
}

impl<'parser> SymbolsTable<'parser> {
    pub fn get_symbols_id(
        &self,
        name: &'parser str,
        span: Span,
    ) -> Result<FoundSymbolId<'parser>, ThrushCompilerIssue> {
        if self.structs.contains_key(name) {
            return Ok((
                Some(name),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        if self.functions.contains_key(name) {
            return Ok((
                None,
                Some(name),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        if self.enums.contains_key(name) {
            return Ok((
                None,
                None,
                Some(name),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        for (idx, scope) in self.statics.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((
                    None,
                    None,
                    None,
                    Some((name, idx)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
            }
        }

        if self.global_statics.contains_key(name) {
            return Ok((
                None,
                None,
                None,
                Some((name, 0)),
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        for (idx, scope) in self.constants.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((
                    None,
                    None,
                    None,
                    None,
                    Some((name, idx)),
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
            }
        }

        if self.global_constants.contains_key(name) {
            return Ok((
                None,
                None,
                None,
                None,
                Some((name, 0)),
                None,
                None,
                None,
                None,
                None,
            ));
        }

        if self.custom_types.contains_key(name) {
            return Ok((
                None,
                None,
                None,
                None,
                None,
                Some(name),
                None,
                None,
                None,
                None,
            ));
        }

        if self.parameters.contains_key(name) {
            return Ok((
                None,
                None,
                None,
                None,
                None,
                None,
                Some(name),
                None,
                None,
                None,
            ));
        }

        if self.asm_functions.contains_key(name) {
            return Ok((
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(name),
                None,
                None,
            ));
        }

        for (idx, scope) in self.llis.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some((name, idx)),
                    None,
                ));
            }
        }

        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some((name, idx)),
                ));
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
        lli_id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&LLISymbol<'parser>, ThrushCompilerIssue> {
        if let Some(lli) = self.llis[scope_idx].get(lli_id) {
            return Ok(lli);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("LLI not found."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct_by_id(
        &self,
        struct_id: &'parser str,
        span: Span,
    ) -> Result<Struct<'parser>, ThrushCompilerIssue> {
        if let Some(structure) = self.structs.get(struct_id).cloned() {
            return Ok(structure);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Struct not found."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_asm_function_by_id(
        &self,
        span: Span,
        asm_func_id: &'parser str,
    ) -> Result<AssemblerFunction<'parser>, ThrushCompilerIssue> {
        if let Some(asm_function) = self.asm_functions.get(asm_func_id).cloned() {
            return Ok(asm_function);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Not found"),
            String::from("Assembler function not found."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_function_by_id(
        &self,
        span: Span,
        func_id: &'parser str,
    ) -> Result<Function<'parser>, ThrushCompilerIssue> {
        if let Some(function) = self.functions.get(func_id).cloned() {
            return Ok(function);
        }

        Err(ThrushCompilerIssue::Error(
            "Not found".into(),
            "Function not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_enum_by_id(
        &self,
        enum_id: &'parser str,
        span: Span,
    ) -> Result<EnumSymbol<'parser>, ThrushCompilerIssue> {
        if let Some(enum_found) = self.enums.get(enum_id).cloned() {
            return Ok(enum_found);
        }

        Err(ThrushCompilerIssue::Error(
            "Not found".into(),
            "Enum not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_custom_type_by_id(
        &self,
        custom_type_id: &'parser str,
        span: Span,
    ) -> Result<CustomTypeSymbol<'parser>, ThrushCompilerIssue> {
        if let Some(custom_type) = self.custom_types.get(custom_type_id).cloned() {
            return Ok(custom_type);
        }

        Err(ThrushCompilerIssue::Error(
            "Not found".into(),
            "Custom type not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_local_by_id(
        &self,
        local_id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&LocalSymbol<'parser>, ThrushCompilerIssue> {
        if let Some(local) = self.locals[scope_idx].get(local_id) {
            return Ok(local);
        }

        Err(ThrushCompilerIssue::Error(
            "Not found".into(),
            "Local not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_static_by_id(
        &self,
        static_id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<StaticSymbol<'parser>, ThrushCompilerIssue> {
        if scope_idx == 0 {
            if let Some(static_var) = self.global_statics.get(static_id).cloned() {
                return Ok(static_var);
            }
        }

        if let Some(scope) = self.statics.get(scope_idx) {
            if let Some(static_var) = scope.get(static_id).cloned() {
                return Ok(static_var);
            }
        } else {
            return Err(ThrushCompilerIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }

        Err(ThrushCompilerIssue::Error(
            "Not found".into(),
            "Static reference not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_const_by_id(
        &self,
        const_id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<ConstantSymbol<'parser>, ThrushCompilerIssue> {
        if scope_idx == 0 {
            if let Some(constant) = self.global_constants.get(const_id).cloned() {
                return Ok(constant);
            }
        }

        if let Some(scope) = self.constants.get(scope_idx) {
            if let Some(local_const) = scope.get(const_id).cloned() {
                return Ok(local_const);
            }
        } else {
            return Err(ThrushCompilerIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }

        Err(ThrushCompilerIssue::Error(
            "Not found".into(),
            "Constant reference not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_parameter_by_id(
        &self,
        parameter_id: &'parser str,
        span: Span,
    ) -> Result<ParameterSymbol<'parser>, ThrushCompilerIssue> {
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
    ) -> Result<Struct<'parser>, ThrushCompilerIssue> {
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
}
