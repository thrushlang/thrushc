use crate::middle::{
    instruction::Instruction,
    statement::{CustomType, Enum},
    symbols::types::{
        Bindings, Constant, Constants, CustomTypes, Enums, Function, Functions, Local, Locals,
        Struct, Structs,
    },
    types::BindingsApplicant,
};

use super::{super::common::error::ThrushCompilerError, lexer::Span};

use ahash::AHashMap as HashMap;

const MINIMAL_CUSTOM_TYPE_CAPACITY: usize = 255;
const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_STRUCTURE_CAPACITY: usize = 255;
const MINIMAL_ENUMS_CAPACITY: usize = 255;
const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 255;

pub type FoundSymbolId<'instr> = (
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<(&'instr str, usize)>,
);

#[derive(Clone, Debug, Default)]
pub struct SymbolsTable<'instr> {
    custom_types: CustomTypes<'instr>,
    constants: Constants<'instr>,
    locals: Locals<'instr>,
    structs: Structs<'instr>,
    functions: Functions<'instr>,
    enums: Enums<'instr>,
}

impl<'instr> SymbolsTable<'instr> {
    pub fn with_functions(functions: HashMap<&'instr str, Function<'instr>>) -> Self {
        Self {
            custom_types: HashMap::with_capacity(MINIMAL_CUSTOM_TYPE_CAPACITY),
            constants: HashMap::with_capacity(MINIMAL_CONSTANTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY),
            functions,
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
            enums: HashMap::with_capacity(MINIMAL_ENUMS_CAPACITY),
        }
    }

    pub fn get_symbols_id(
        &self,
        name: &'instr str,
        span: Span,
    ) -> Result<FoundSymbolId<'instr>, ThrushCompilerError> {
        if self.custom_types.contains_key(name) {
            return Ok((None, None, None, None, Some(name), None));
        }

        if self.constants.contains_key(name) {
            return Ok((None, None, None, Some(name), None, None));
        }

        if self.structs.contains_key(name) {
            return Ok((Some(name), None, None, None, None, None));
        }

        if self.enums.contains_key(name) {
            return Ok((None, None, Some(name), None, None, None));
        }

        if self.functions.contains_key(name) {
            return Ok((None, Some(name), None, None, None, None));
        }

        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((None, None, None, None, None, Some((name, idx))));
            }
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure/Function/Local/Constant/Type not found"),
            format!("'{}' is not declared or defined.", name),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_struct_by_id(
        &self,
        struct_id: &'instr str,
        span: Span,
    ) -> Result<Struct<'instr>, ThrushCompilerError> {
        if let Some(structure) = self.structs.get(struct_id).cloned() {
            return Ok(structure);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_function_by_id(
        &self,
        span: Span,
        func_id: &'instr str,
    ) -> Result<Function<'instr>, ThrushCompilerError> {
        if let Some(function) = self.functions.get(func_id).cloned() {
            return Ok(function);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_enum_by_id(
        &self,
        enum_id: &'instr str,
        span: Span,
    ) -> Result<Enum<'instr>, ThrushCompilerError> {
        if let Some(enum_found) = self.enums.get(enum_id).cloned() {
            return Ok(enum_found);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_custom_type_by_id(
        &self,
        custom_type_id: &'instr str,
        span: Span,
    ) -> Result<CustomType<'instr>, ThrushCompilerError> {
        if let Some(custom_type) = self.custom_types.get(custom_type_id).cloned() {
            return Ok(custom_type);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_local_by_id(
        &self,
        local_id: &'instr str,
        scope_idx: usize,
        span: Span,
    ) -> Result<&Local<'instr>, ThrushCompilerError> {
        if let Some(local) = self.locals[scope_idx].get(local_id) {
            return Ok(local);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_const_by_id(
        &self,
        const_id: &'instr str,
        span: Span,
    ) -> Result<Constant<'instr>, ThrushCompilerError> {
        if let Some(constant) = self.constants.get(const_id).cloned() {
            return Ok(constant);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_struct(
        &self,
        name: &str,
        span: Span,
    ) -> Result<Struct<'instr>, ThrushCompilerError> {
        if let Some(struct_fields) = self.structs.get(name).cloned() {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            String::default(),
            span,
        ))
    }

    #[inline]
    pub fn get_struct_mut(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<&mut Struct<'instr>, ThrushCompilerError> {
        if let Some(struct_fields) = self.structs.get_mut(name) {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            String::default(),
            span,
        ))
    }

    pub fn new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: Local<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerError> {
        if self
            .locals
            .iter()
            .rev()
            .any(|scope| scope.contains_key(name))
        {
            return Err(ThrushCompilerError::Error(
                String::from("Local variable already declared"),
                format!("'{}' local variable already declared before.", name),
                String::default(),
                span,
            ));
        }

        self.locals[scope_pos - 1].insert(name, value);

        Ok(())
    }

    pub fn new_constant(
        &mut self,
        name: &'instr str,
        constant: Constant<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerError> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Constant already declared"),
                format!("'{}' constant already declared before.", name),
                String::default(),
                span,
            ));
        }

        self.constants.insert(name, constant);

        Ok(())
    }

    pub fn new_custom_type(
        &mut self,
        name: &'instr str,
        custom_type: CustomType<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerError> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Custom type already declared"),
                format!("'{}' custom type already declared before.", name),
                String::default(),
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
    ) -> Result<(), ThrushCompilerError> {
        if self.structs.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Structure already declared"),
                format!("'{}' structure already declared before.", name),
                String::default(),
                span,
            ));
        }

        self.structs.insert(name, field_types);

        Ok(())
    }

    pub fn new_enum(
        &mut self,
        name: &'instr str,
        union: Enum<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerError> {
        if self.enums.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Enum already declared"),
                format!("'{}' enum already declared before.", name),
                String::default(),
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
    ) -> Result<(), ThrushCompilerError> {
        if self.functions.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Function already declared"),
                format!("'{}' function already declared before.", name),
                String::default(),
                span,
            ));
        }

        self.functions.insert(name, function);

        Ok(())
    }

    pub fn set_bindings(
        &mut self,
        name: &str,
        bindings: Bindings<'instr>,
        applicant: BindingsApplicant,
        span: Span,
    ) -> Result<(), ThrushCompilerError> {
        match applicant {
            BindingsApplicant::Struct => {
                let structure: &mut Struct = self.get_struct_mut(name, span)?;
                structure.3 = bindings;
            }
        }

        Ok(())
    }

    pub fn contains_structure(&self, name: &str, span: Span) -> Result<(), ThrushCompilerError> {
        if !self.structs.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Structure not found"),
                format!("'{}' structure not defined or declared yet.", name),
                String::default(),
                span,
            ));
        }

        Ok(())
    }

    pub fn lift(
        &mut self,
        scope_pos: usize,
        locals: &mut Vec<Instruction<'instr>>,
    ) -> Result<(), ThrushCompilerError> {
        for parameter in &*locals {
            if let Instruction::FunctionParameter {
                name,
                kind,
                is_mutable,
                span,
                ..
            }
            | Instruction::BindParameter {
                name,
                kind,
                is_mutable,
                span,
                ..
            } = parameter
            {
                self.new_local(
                    scope_pos,
                    name,
                    (kind.clone(), *is_mutable, false, *span),
                    *span,
                )?;
            }

            if let Instruction::Local {
                name,
                kind,
                span,
                is_mutable,
                ..
            } = parameter
            {
                self.new_local(
                    scope_pos,
                    name,
                    (kind.clone(), *is_mutable, false, *span),
                    *span,
                )?;
            }
        }

        locals.clear();
        Ok(())
    }

    #[inline]
    pub fn begin_local_scope(&mut self) {
        self.locals
            .push(HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY));
    }

    #[inline]
    pub fn end_local_scope(&mut self) {
        self.locals.pop();
    }
}
