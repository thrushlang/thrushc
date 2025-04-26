use crate::middle::statement::{CustomType, Enum, ThrushAttributes};

use super::{super::common::error::ThrushCompilerError, super::middle::types::Type, lexer::Span};

use ahash::AHashMap as HashMap;

const MINIMAL_CUSTOM_TYPE_CAPACITY: usize = 255;
const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_STRUCTURE_CAPACITY: usize = 255;
const MINIMAL_ENUMS_CAPACITY: usize = 255;
const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 255;

pub type Constant<'instr> = (Type, ThrushAttributes<'instr>);

pub type Struct<'instr> = (Vec<(&'instr str, Type, u32)>, ThrushAttributes<'instr>);

pub type Function<'instr> = (Type, Vec<Type>, bool);
pub type Local<'instr> = (Type, bool, bool);

pub type CustomTypes<'instr> = HashMap<&'instr str, CustomType<'instr>>;
pub type Constants<'instr> = HashMap<&'instr str, Constant<'instr>>;

pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, Enum<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;

pub type Locals<'instr> = Vec<HashMap<&'instr str, Local<'instr>>>;

pub type FoundObjectId<'instr> = (
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<(&'instr str, usize)>,
);

#[derive(Clone, Debug, Default)]
pub struct ParserObjects<'instr> {
    custom_types: CustomTypes<'instr>,
    constants: Constants<'instr>,
    locals: Locals<'instr>,
    structs: Structs<'instr>,
    functions: Functions<'instr>,
    enums: Enums<'instr>,
}

impl<'instr> ParserObjects<'instr> {
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

    pub fn get_object_id(
        &self,
        name: &'instr str,
        span: Span,
    ) -> Result<FoundObjectId<'instr>, ThrushCompilerError> {
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
            span,
        ))
    }

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
            span,
        ))
    }

    #[inline]
    pub fn new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: Local<'instr>,
        span: Span,
    ) -> Result<(), ThrushCompilerError> {
        if self.locals[scope_pos - 1].contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Local variable already declared"),
                format!("'{}' local variable already declared before.", name),
                span,
            ));
        }

        self.locals[scope_pos - 1].insert(name, value);

        Ok(())
    }

    #[inline]
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
                span,
            ));
        }

        self.constants.insert(name, constant);

        Ok(())
    }

    #[inline]
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
                span,
            ));
        }

        self.custom_types.insert(name, custom_type);

        Ok(())
    }

    #[inline]
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
                span,
            ));
        }

        self.structs.insert(name, field_types);

        Ok(())
    }

    #[inline]
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
                span,
            ));
        }

        self.enums.insert(name, union);

        Ok(())
    }

    #[inline]
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
                span,
            ));
        }

        self.functions.insert(name, function);

        Ok(())
    }

    #[inline]
    pub fn begin_local_scope(&mut self) {
        self.locals
            .push(HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY));
    }

    #[inline(always)]
    pub fn end_local_scope(&mut self) {
        self.locals.pop();
    }
}
