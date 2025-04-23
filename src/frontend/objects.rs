use super::super::{
    backend::compiler::instruction::Instruction,
    backend::compiler::types::{Enum, ThrushAttributes},
    common::error::ThrushCompilerError,
};

use super::types::CodeLocation;

use ahash::AHashMap as HashMap;

const MINIMAL_STRUCTURE_CAPACITY: usize = 200;
const MINIMAL_ENUMS_CAPACITY: usize = 200;

const MINIMAL_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 255;

pub type Struct<'instr> = (
    Vec<(&'instr str, Instruction<'instr>, u32)>,
    ThrushAttributes<'instr>,
);

pub type Function<'instr> = (Instruction<'instr>, Vec<Instruction<'instr>>, bool);

pub type Constant<'instr> = (Instruction<'instr>, ThrushAttributes<'instr>);
pub type Local<'instr> = (Instruction<'instr>, bool, bool);

pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Enums<'instr> = HashMap<&'instr str, Enum<'instr>>;
pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;

pub type Constants<'instr> = HashMap<&'instr str, Constant<'instr>>;
pub type Locals<'instr> = Vec<HashMap<&'instr str, Local<'instr>>>;

pub type FoundObjectId<'instr> = (
    Option<&'instr str>,
    Option<&'instr str>,
    Option<&'instr str>,
    Option<(&'instr str, usize)>,
);

#[derive(Clone, Debug, Default)]
pub struct ParserObjects<'instr> {
    constants: Constants<'instr>,
    locals: Locals<'instr>,
    structs: Structs<'instr>,
    functions: Functions<'instr>,
    enums: Enums<'instr>,
}

impl<'instr> ParserObjects<'instr> {
    pub fn with_functions(functions: HashMap<&'instr str, Function<'instr>>) -> Self {
        Self {
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
        location: CodeLocation,
    ) -> Result<FoundObjectId<'instr>, ThrushCompilerError> {
        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((None, None, None, Some((name, idx))));
            }
        }

        if self.structs.contains_key(name) {
            return Ok((Some(name), None, None, None));
        }

        if self.enums.contains_key(name) {
            return Ok((None, None, Some(name), None));
        }

        if self.functions.contains_key(name) {
            return Ok((None, Some(name), None, None));
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure/Function/Local not found"),
            format!("'{}' is not declared or defined.", name),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn get_function_by_id(
        &self,
        location: CodeLocation,
        func_id: &'instr str,
    ) -> Result<Function<'instr>, ThrushCompilerError> {
        if let Some(function) = self.functions.get(func_id).cloned() {
            return Ok(function);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn get_enum_by_id(
        &self,
        enum_id: &'instr str,
        location: CodeLocation,
    ) -> Result<Enum<'instr>, ThrushCompilerError> {
        if let Some(enum_found) = self.enums.get(enum_id).cloned() {
            return Ok(enum_found);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn get_local_by_id(
        &self,
        local_id: &'instr str,
        scope_idx: usize,
        location: CodeLocation,
    ) -> Result<&Local<'instr>, ThrushCompilerError> {
        if let Some(local) = self.locals[scope_idx].get(local_id) {
            return Ok(local);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn get_struct(
        &self,
        name: &str,
        location: CodeLocation,
    ) -> Result<Struct<'instr>, ThrushCompilerError> {
        if let Some(struct_fields) = self.structs.get(name).cloned() {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure not found"),
            format!("'{}' structure not defined.", name),
            location.0,
            Some(location.1),
        ))
    }

    #[inline]
    pub fn insert_new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: Local<'instr>,
        location: CodeLocation,
    ) -> Result<(), ThrushCompilerError> {
        if self.locals[scope_pos - 1].contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Local variable already declared"),
                format!("'{}' local variable already declared.", name),
                location.0,
                Some(location.1),
            ));
        }

        self.locals[scope_pos - 1].insert(name, value);

        Ok(())
    }

    #[inline]
    pub fn insert_new_constant(
        &mut self,
        name: &'instr str,
        constant: Constant<'instr>,
        location: CodeLocation,
    ) -> Result<(), ThrushCompilerError> {
        if self.constants.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Constant already declared"),
                format!("'{}' constant already declared.", name),
                location.0,
                Some(location.1),
            ));
        }

        self.constants.insert(name, constant);

        Ok(())
    }

    #[inline]
    pub fn insert_new_struct(
        &mut self,
        name: &'instr str,
        field_types: Struct<'instr>,
        location: CodeLocation,
    ) -> Result<(), ThrushCompilerError> {
        if self.structs.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Structure already declared"),
                format!("'{}' structure already declared before.", name),
                location.0,
                Some(location.1),
            ));
        }

        self.structs.insert(name, field_types);

        Ok(())
    }

    #[inline]
    pub fn insert_new_enum(
        &mut self,
        name: &'instr str,
        union: Enum<'instr>,
        location: CodeLocation,
    ) -> Result<(), ThrushCompilerError> {
        if self.enums.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Enum already declared"),
                format!("'{}' enum already declared before.", name),
                location.0,
                Some(location.1),
            ));
        }

        self.enums.insert(name, union);

        Ok(())
    }

    #[inline]
    pub fn insert_new_function(
        &mut self,
        name: &'instr str,
        function: Function<'instr>,
        location: CodeLocation,
    ) -> Result<(), ThrushCompilerError> {
        if self.functions.contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Function already declared"),
                format!("'{}' function already declared before.", name),
                location.0,
                Some(location.1),
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
