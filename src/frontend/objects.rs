use super::super::backend::compiler::instruction::Instruction;
use super::super::common::error::ThrushCompilerError;
use super::types::CodeLocation;
use ahash::AHashMap as HashMap;

const MINIMAL_STRUCTURE_CAPACITY: usize = 1024;
const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 255;

pub type Function<'instr> = (Instruction<'instr>, Vec<Instruction<'instr>>, bool);

pub type Struct<'instr> = Vec<(&'instr str, Instruction<'instr>, u32)>;
pub type Local<'instr> = (Instruction<'instr>, bool, bool);

pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;
pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Locals<'instr> = Vec<HashMap<&'instr str, Local<'instr>>>;

pub type FoundObjectId<'instr> = (
    Option<&'instr str>,
    Option<&'instr str>,
    Option<(&'instr str, usize)>,
);

#[derive(Clone, Debug, Default)]
pub struct ParserObjects<'instr> {
    locals: Locals<'instr>,
    functions: Functions<'instr>,
    structs: Structs<'instr>,
}

impl<'instr> ParserObjects<'instr> {
    pub fn with_functions(functions: HashMap<&'instr str, Function<'instr>>) -> Self {
        Self {
            locals: Vec::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY),
            functions,
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
        }
    }

    pub fn get_object_id(
        &self,
        name: &'instr str,
        location: CodeLocation,
    ) -> Result<FoundObjectId<'instr>, ThrushCompilerError> {
        for (idx, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Ok((None, None, Some((name, idx))));
            }
        }

        if self.functions.contains_key(name) {
            return Ok((None, Some(name), None));
        }

        if self.structs.contains_key(name) {
            return Ok((Some(name), None, None));
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure/Function/Local not found"),
            format!("'{}' is not defined.", name),
            location.0,
            Some(location.1),
        ))
    }

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

    pub fn get_local_by_id(
        &self,
        location: CodeLocation,
        local_id: &'instr str,
        scope_idx: usize,
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

    pub fn get_struct(
        &self,
        name: &str,
        location: (usize, (usize, usize)),
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

    #[inline(always)]
    pub fn insert_new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: Local<'instr>,
        line: usize,
        span: (usize, usize),
    ) -> Result<(), ThrushCompilerError> {
        if self.locals[scope_pos - 1].contains_key(name) {
            return Err(ThrushCompilerError::Error(
                String::from("Local variable already declared"),
                format!("'{}' local variable already declared.", name),
                line,
                Some(span),
            ));
        }

        self.locals[scope_pos - 1].insert(name, value);

        Ok(())
    }

    #[inline(always)]
    pub fn insert_new_struct(&mut self, name: &'instr str, field_types: Struct<'instr>) {
        if self.structs.contains_key(name) {
            return;
        }

        self.structs.insert(name, field_types);
    }

    #[inline(always)]
    pub fn insert_new_function(&mut self, name: &'instr str, function: Function<'instr>) {
        if self.functions.contains_key(name) {
            return;
        }

        self.functions.insert(name, function);
    }

    #[inline(always)]
    pub fn begin_local_scope(&mut self) {
        self.locals
            .push(HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY));
    }

    #[inline(always)]
    pub fn end_local_scope(&mut self) {
        self.locals.pop();
    }
}
