use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::parser::constants::PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY;
use crate::front_end::parser::constants::PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::symbols::types::AssemblerFunction;
use crate::front_end::types::parser::symbols::types::AssemblerFunctions;
use crate::front_end::types::parser::symbols::types::ConstantSymbol;
use crate::front_end::types::parser::symbols::types::CustomTypeSymbol;
use crate::front_end::types::parser::symbols::types::EnumSymbol;
use crate::front_end::types::parser::symbols::types::FoundSymbolId;
use crate::front_end::types::parser::symbols::types::Function;
use crate::front_end::types::parser::symbols::types::Functions;
use crate::front_end::types::parser::symbols::types::GlobalConstants;
use crate::front_end::types::parser::symbols::types::GlobalCustomTypes;
use crate::front_end::types::parser::symbols::types::GlobalEnums;
use crate::front_end::types::parser::symbols::types::GlobalStatics;
use crate::front_end::types::parser::symbols::types::GlobalStructs;
use crate::front_end::types::parser::symbols::types::Intrinsic;
use crate::front_end::types::parser::symbols::types::Intrinsics;
use crate::front_end::types::parser::symbols::types::LLISymbol;
use crate::front_end::types::parser::symbols::types::LLIs;
use crate::front_end::types::parser::symbols::types::LocalConstants;
use crate::front_end::types::parser::symbols::types::LocalCustomTypes;
use crate::front_end::types::parser::symbols::types::LocalEnums;
use crate::front_end::types::parser::symbols::types::LocalStatics;
use crate::front_end::types::parser::symbols::types::LocalStructs;
use crate::front_end::types::parser::symbols::types::LocalSymbol;
use crate::front_end::types::parser::symbols::types::Locals;
use crate::front_end::types::parser::symbols::types::ParameterSymbol;
use crate::front_end::types::parser::symbols::types::Parameters;
use crate::front_end::types::parser::symbols::types::StaticSymbol;
use crate::front_end::types::parser::symbols::types::Struct;

use ahash::AHashMap as HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct SymbolsTable<'parser> {
    functions: Functions<'parser>,
    asm_functions: AssemblerFunctions<'parser>,
    intrinsics: Intrinsics<'parser>,

    global_custom_types: GlobalCustomTypes<'parser>,
    global_statics: GlobalStatics<'parser>,
    global_structs: GlobalStructs<'parser>,
    global_constants: GlobalConstants<'parser>,
    global_enums: GlobalEnums<'parser>,

    local_structs: LocalStructs<'parser>,
    local_statics: LocalStatics<'parser>,
    local_constants: LocalConstants<'parser>,
    local_custom_types: LocalCustomTypes<'parser>,
    local_enums: LocalEnums<'parser>,

    locals: Locals<'parser>,
    llis: LLIs<'parser>,
    parameters: Parameters<'parser>,
}

impl<'parser> SymbolsTable<'parser> {
    pub fn with_functions(
        functions: Functions<'parser>,
        asm_functions: AssemblerFunctions<'parser>,
    ) -> Self {
        Self {
            functions,
            asm_functions,

            intrinsics: HashMap::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),

            global_structs: HashMap::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),
            global_statics: HashMap::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),
            global_constants: HashMap::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),
            global_custom_types: HashMap::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),
            global_enums: HashMap::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),

            local_structs: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY),
            local_statics: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY),
            local_constants: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY),
            local_custom_types: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),
            local_enums: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_GLOBAL_CAPACITY),
            locals: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY),
            llis: Vec::with_capacity(PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY),

            parameters: HashMap::with_capacity(10),
        }
    }
}

impl SymbolsTable<'_> {
    #[inline]
    pub fn begin_scope(&mut self) {
        self.local_structs.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));
        self.local_custom_types.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));
        self.local_statics.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));
        self.local_constants.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));
        self.local_enums.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));

        self.locals.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));
        self.llis.push(HashMap::with_capacity(
            PARSER_SYMBOLS_MINIMAL_LOCAL_CAPACITY,
        ));
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.local_statics.pop();
        self.local_constants.pop();
        self.local_structs.pop();
        self.local_custom_types.pop();
        self.local_enums.pop();

        self.locals.pop();
        self.llis.pop();
    }

    #[inline]
    pub fn finish_scopes(&mut self) {
        self.local_statics.clear();
        self.local_constants.clear();
        self.local_structs.clear();
        self.local_custom_types.clear();
        self.local_enums.clear();

        self.locals.clear();
        self.llis.clear();
    }

    #[inline]
    pub fn finish_parameters(&mut self) {
        self.parameters.clear();
    }
}

impl<'parser> SymbolsTable<'parser> {
    pub fn declare_parameters(
        &mut self,
        parameters: &[Ast<'parser>],
    ) -> Result<(), CompilationIssue> {
        parameters.iter().try_for_each(|parameter| {
            if let Ast::FunctionParameter {
                name: id,
                kind,
                span,
                metadata,
                ..
            } = parameter
            {
                if self.parameters.contains_key(id) {
                    return Err(CompilationIssue::Error(
                        "Parameter already declared".into(),
                        format!("'{}' parameter already declared before.", id),
                        None,
                        *span,
                    ));
                }

                self.parameters.insert(id, (kind.clone(), *metadata, *span));
            }

            Ok(())
        })?;

        Ok(())
    }
}

impl<'parser> SymbolsTable<'parser> {
    pub fn new_lli(
        &mut self,
        id: &'parser str,
        lli: LLISymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.llis.last_mut() {
            if last_scope.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Low level instruction already declared".into(),
                    format!("Low level instruction '{}' already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, lli);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Low level instruction not caught"),
            String::from("The final scope was not obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_local(
        &mut self,
        id: &'parser str,
        local: LocalSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.locals.last_mut() {
            if last_scope.contains_key(id) || self.parameters.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Local variable already declared".into(),
                    format!("'{}' local variable already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, local);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_global_static(
        &mut self,
        id: &'parser str,
        static_: StaticSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.global_statics.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Static already declared".into(),
                format!("'{}' static already declared before.", id),
                None,
                span,
            ));
        }

        self.global_statics.insert(id, static_);

        Ok(())
    }

    pub fn new_static(
        &mut self,
        id: &'parser str,
        static_: StaticSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.local_statics.last_mut() {
            if last_scope.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Static already declared".into(),
                    format!("'{}' static already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, static_);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_global_constant(
        &mut self,
        id: &'parser str,
        constant: ConstantSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.global_constants.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Constant already declared".into(),
                format!("'{}' constant already declared before.", id),
                None,
                span,
            ));
        }

        self.global_constants.insert(id, constant);

        Ok(())
    }

    pub fn new_constant(
        &mut self,
        id: &'parser str,
        constant: ConstantSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.local_constants.last_mut() {
            if last_scope.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Constant already declared".into(),
                    format!("'{}' constant already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, constant);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_global_custom_type(
        &mut self,
        id: &'parser str,
        ctype: CustomTypeSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.global_custom_types.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Custom type already declared".into(),
                format!("'{}' custom type already declared before.", id),
                None,
                span,
            ));
        }

        self.global_custom_types.insert(id, ctype);

        Ok(())
    }

    pub fn new_custom_type(
        &mut self,
        id: &'parser str,
        ctype: CustomTypeSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.local_custom_types.last_mut() {
            if last_scope.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Custom already declared".into(),
                    format!("'{}' Custom already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, ctype);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_global_struct(
        &mut self,
        id: &'parser str,
        fields: Struct<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.global_structs.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Structure already declared".into(),
                format!("'{}' structure already declared before.", id),
                None,
                span,
            ));
        }

        self.global_structs.insert(id, fields);

        Ok(())
    }

    pub fn new_struct(
        &mut self,
        id: &'parser str,
        fields: Struct<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.local_structs.last_mut() {
            if last_scope.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Structure already declared".into(),
                    format!("'{}' structure already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, fields);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_global_enum(
        &mut self,
        id: &'parser str,
        union: EnumSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.global_enums.contains_key(id) {
            return Err(CompilationIssue::Error(
                String::from("Enum already declared"),
                format!("'{}' enum already declared before.", id),
                None,
                span,
            ));
        }

        self.global_enums.insert(id, union);

        Ok(())
    }

    pub fn new_enum(
        &mut self,
        id: &'parser str,
        union: EnumSymbol<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if let Some(last_scope) = self.local_enums.last_mut() {
            if last_scope.contains_key(id) {
                return Err(CompilationIssue::Error(
                    "Enum already declared".into(),
                    format!("'{}' enum already declared before.", id),
                    None,
                    span,
                ));
            }

            last_scope.insert(id, union);

            return Ok(());
        }

        return Err(CompilationIssue::FrontEndBug(
            String::from("Last scope not caught"),
            String::from("The last scope could not be obtained."),
            span,
            CompilationPosition::Parser,
            PathBuf::from(file!()),
            line!(),
        ));
    }

    pub fn new_asm_function(
        &mut self,
        id: &'parser str,
        function: AssemblerFunction<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.asm_functions.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Assembly function already declared".into(),
                format!("'{}' assembler function already declared before.", id),
                None,
                span,
            ));
        }

        self.asm_functions.insert(id, function);

        Ok(())
    }

    pub fn new_function(
        &mut self,
        id: &'parser str,
        function: Function<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.functions.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Function already declared".into(),
                format!("'{}' function already declared before.", id),
                None,
                span,
            ));
        }

        self.functions.insert(id, function);

        Ok(())
    }

    pub fn new_intrinsic(
        &mut self,
        id: &'parser str,
        intrinsic: Intrinsic<'parser>,
        span: Span,
    ) -> Result<(), CompilationIssue> {
        if self.intrinsics.contains_key(id) {
            return Err(CompilationIssue::Error(
                "Intrinsic already declared".into(),
                format!("'{}' intrinsic already declared before.", id),
                None,
                span,
            ));
        }

        self.intrinsics.insert(id, intrinsic);

        Ok(())
    }
}

impl<'parser> SymbolsTable<'parser> {
    pub fn get_symbols_id(
        &self,
        id: &'parser str,
        span: Span,
    ) -> Result<FoundSymbolId<'parser>, CompilationIssue> {
        for (idx, scope) in self.llis.iter().enumerate().rev() {
            if scope.contains_key(id) {
                return Ok((
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some((id, idx)),
                    None,
                    None,
                ));
            }
        }

        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(id) {
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
                    Some((id, idx)),
                    None,
                ));
            }
        }

        if self.parameters.contains_key(id) {
            return Ok((
                None,
                None,
                None,
                None,
                None,
                None,
                Some(id),
                None,
                None,
                None,
                None,
            ));
        }

        for (idx, scope) in self.local_structs.iter().enumerate().rev() {
            if scope.contains_key(id) {
                return Ok((
                    Some((id, idx)),
                    None,
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
        }
        if self.global_structs.contains_key(id) {
            return Ok((
                Some((id, 0)),
                None,
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

        for (idx, scope) in self.local_enums.iter().enumerate().rev() {
            if scope.contains_key(id) {
                return Ok((
                    None,
                    None,
                    Some((id, idx)),
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
        }
        if self.global_enums.contains_key(id) {
            return Ok((
                None,
                None,
                Some((id, 0)),
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

        for (idx, scope) in self.local_custom_types.iter().enumerate().rev() {
            if scope.contains_key(id) {
                return Ok((
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some((id, idx)),
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
            }
        }
        if self.global_custom_types.contains_key(id) {
            return Ok((
                None,
                None,
                None,
                None,
                None,
                Some((id, 0)),
                None,
                None,
                None,
                None,
                None,
            ));
        }

        if self.intrinsics.contains_key(id) {
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
                None,
                Some(id),
            ));
        }

        if self.functions.contains_key(id) {
            return Ok((
                None,
                Some(id),
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

        if self.asm_functions.contains_key(id) {
            return Ok((
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(id),
                None,
                None,
                None,
            ));
        }

        for (idx, scope) in self.local_constants.iter().enumerate().rev() {
            if scope.contains_key(id) {
                return Ok((
                    None,
                    None,
                    None,
                    None,
                    Some((id, idx)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
            }
        }

        if self.global_constants.contains_key(id) {
            return Ok((
                None,
                None,
                None,
                None,
                Some((id, 0)),
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        for (idx, scope) in self.local_statics.iter().enumerate().rev() {
            if scope.contains_key(id) {
                return Ok((
                    None,
                    None,
                    None,
                    Some((id, idx)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
            }
        }
        if self.global_statics.contains_key(id) {
            return Ok((
                None,
                None,
                None,
                Some((id, 0)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        Err(CompilationIssue::Error(
            String::from("Not found"),
            format!("'{}' isn't declared or defined.", id),
            None,
            span,
        ))
    }
}

impl<'parser> SymbolsTable<'parser> {
    #[inline]
    pub fn get_lli_by_id(
        &self,
        id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&LLISymbol<'parser>, CompilationIssue> {
        if let Some(scope) = self.llis.get(scope_idx) {
            if let Some(lli) = scope.get(id) {
                return Ok(lli);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Scope not caught"),
                String::from("The scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
            String::from("Not found"),
            String::from("LLI not found."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_asm_function_by_id(
        &self,
        span: Span,
        id: &'parser str,
    ) -> Result<AssemblerFunction<'parser>, CompilationIssue> {
        if let Some(asm_function) = self.asm_functions.get(id).cloned() {
            return Ok(asm_function);
        }

        Err(CompilationIssue::Error(
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
        id: &'parser str,
    ) -> Result<Function<'parser>, CompilationIssue> {
        if let Some(function) = self.functions.get(id).cloned() {
            return Ok(function);
        }

        Err(CompilationIssue::Error(
            "Not found".into(),
            "Function not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_intrinsic_by_id(
        &self,
        span: Span,
        id: &'parser str,
    ) -> Result<Intrinsic<'parser>, CompilationIssue> {
        if let Some(intrinsic) = self.intrinsics.get(id).cloned() {
            return Ok(intrinsic);
        }

        Err(CompilationIssue::Error(
            "Not found".into(),
            "Intrinsic not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_enum_by_id(
        &self,
        id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<EnumSymbol<'parser>, CompilationIssue> {
        if scope_idx == 0 {
            if let Some(lenum) = self.global_enums.get(id).cloned() {
                return Ok(lenum);
            }
        }

        if let Some(scope) = self.local_enums.get(scope_idx) {
            if let Some(lenum) = scope.get(id).cloned() {
                return Ok(lenum);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
            "Not found".into(),
            "Enum reference not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_custom_type_by_id(
        &self,
        id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<CustomTypeSymbol<'parser>, CompilationIssue> {
        if scope_idx == 0 {
            if let Some(ctype) = self.global_custom_types.get(id).cloned() {
                return Ok(ctype);
            }
        }

        if let Some(scope) = self.local_custom_types.get(scope_idx) {
            if let Some(ctype) = scope.get(id).cloned() {
                return Ok(ctype);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
            "Not found".into(),
            "Custom type reference not found.".into(),
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
    ) -> Result<&LocalSymbol<'parser>, CompilationIssue> {
        if let Some(scope) = self.locals.get(scope_idx) {
            if let Some(local) = scope.get(local_id) {
                return Ok(local);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Scope not caught"),
                String::from("The scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
            "Not found".into(),
            "Local not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_static_by_id(
        &self,
        id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<StaticSymbol<'parser>, CompilationIssue> {
        if scope_idx == 0 {
            if let Some(static_var) = self.global_statics.get(id).cloned() {
                return Ok(static_var);
            }
        }

        if let Some(scope) = self.local_statics.get(scope_idx) {
            if let Some(static_var) = scope.get(id).cloned() {
                return Ok(static_var);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
            "Not found".into(),
            "Static reference not found.".into(),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_const_by_id(
        &self,
        id: &'parser str,
        scope_idx: usize,
        span: Span,
    ) -> Result<ConstantSymbol<'parser>, CompilationIssue> {
        if scope_idx == 0 {
            if let Some(constant) = self.global_constants.get(id).cloned() {
                return Ok(constant);
            }
        }

        if let Some(scope) = self.local_constants.get(scope_idx) {
            if let Some(local_const) = scope.get(id).cloned() {
                return Ok(local_const);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
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
    ) -> Result<ParameterSymbol<'parser>, CompilationIssue> {
        if let Some(parameter) = self.parameters.get(parameter_id).cloned() {
            return Ok(parameter);
        }

        Err(CompilationIssue::Error(
            String::from("Not found"),
            String::from("Parameter not found in this scope."),
            None,
            span,
        ))
    }

    #[inline]
    pub fn get_struct_by_id(
        &self,
        id: &str,
        scope_idx: usize,
        span: Span,
    ) -> Result<Struct<'parser>, CompilationIssue> {
        if scope_idx == 0 {
            if let Some(structure) = self.global_structs.get(id).cloned() {
                return Ok(structure);
            }
        }

        if let Some(scope) = self.local_structs.get(scope_idx) {
            if let Some(local_struct) = scope.get(id).cloned() {
                return Ok(local_struct);
            }
        } else {
            return Err(CompilationIssue::FrontEndBug(
                String::from("Last scope not caught"),
                String::from("The last scope could not be obtained."),
                span,
                CompilationPosition::Parser,
                PathBuf::from(file!()),
                line!(),
            ));
        }

        Err(CompilationIssue::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", id),
            None,
            span,
        ))
    }
}
