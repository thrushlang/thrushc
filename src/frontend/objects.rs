use {
    super::{
        super::{backend::instruction::Instruction, error::ThrushCompilerError},
        lexer::Type,
        traits::FoundObjectEither,
    },
    ahash::AHashMap as HashMap,
};

const MINIMAL_STRUCTURE_CAPACITY: usize = 1024;
const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 255;
const MINIMAL_DEALLOCATORS_CAPACITY: usize = 50;

pub type Function<'instr> = (Type, Vec<Type>, Vec<(String, usize)>, String, bool);

pub type Struct<'instr> = HashMap<&'instr str, Type>;
pub type Local = (Type, String, bool, bool);

pub type Functions<'instr> = HashMap<&'instr str, Function<'instr>>;
pub type Structs<'instr> = HashMap<&'instr str, Struct<'instr>>;
pub type Locals<'instr> = Vec<HashMap<&'instr str, Local>>;

pub type FoundObject<'instr> = (
    Option<&'instr Struct<'instr>>,
    Option<&'instr Function<'instr>>,
    Option<&'instr Local>,
);

#[derive(Clone, Debug, Default)]
pub struct ParserObjects<'instr> {
    locals: Locals<'instr>,
    functions: Functions<'instr>,
    structs: Structs<'instr>,
}

impl<'instr> ParserObjects<'instr> {
    pub fn with_functions(functions: HashMap<&'instr str, Function>) -> Self {
        Self {
            locals: vec![HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY)],
            functions,
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
        }
    }

    pub fn get_object(
        &self,
        name: &'instr str,
        location: (usize, (usize, usize)),
    ) -> Result<FoundObject, ThrushCompilerError> {
        for scope in self.locals.iter().rev() {
            if let Some(local) = scope.get(name) {
                return Ok((None, None, Some(local)));
            }
        }

        if let Some(function) = self.functions.get(name) {
            return Ok((None, Some(function), None));
        }

        if let Some(structure) = self.structs.get(name) {
            return Ok((Some(structure), None, None));
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure/function/local variable not found"),
            format!("'{}' is don't in declared or defined.", name),
            location.0,
            Some(location.1),
        ))
    }

    pub fn get_struct(
        &self,
        name: &str,
        location: (usize, (usize, usize)),
    ) -> Result<HashMap<&'instr str, Type>, ThrushCompilerError> {
        if let Some(struct_fields) = self.structs.get(name).cloned() {
            return Ok(struct_fields);
        }

        Err(ThrushCompilerError::Error(
            String::from("Structure don't found"),
            format!("'{}' structure not declared or defined.", name),
            location.0,
            Some(location.1),
        ))
    }

    #[inline(always)]
    pub fn insert_new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: Local,
        line: usize,
        span: (usize, usize),
        parser_errors: &mut Vec<ThrushCompilerError>,
    ) {
        if self.locals[scope_pos].contains_key(name) {
            parser_errors.push(ThrushCompilerError::Error(
                String::from("Local variable already declared"),
                format!("'{}' local variable already declared.", name),
                line,
                Some(span),
            ));

            return;
        }

        self.locals[scope_pos].insert(name, value);
    }

    #[inline(always)]
    pub fn insert_new_struct(&mut self, name: &'instr str, value: HashMap<&'instr str, Type>) {
        self.structs.insert(name, value);
    }

    #[inline(always)]
    pub fn contains_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    #[inline(always)]
    pub fn insert_new_function(&mut self, name: &'instr str, value: Function) {
        self.functions.insert(name, value);
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

    pub fn modify_local_deallocation(
        &mut self,
        at_scope_pos: usize,
        name: &'instr str,
        mark_as_freeded: bool,
    ) {
        let scope: &mut HashMap<&str, Local> = self.locals.get_mut(at_scope_pos).unwrap();

        if let Some(local) = scope.get(name) {
            let mut local: (Type, String, bool, bool) = local.clone();

            local.2 = mark_as_freeded;

            scope.insert(name, local);
        }
    }

    pub fn create_deallocators(&self, at_scope_pos: usize) -> Vec<Instruction<'instr>> {
        let mut frees: Vec<Instruction> = Vec::with_capacity(MINIMAL_DEALLOCATORS_CAPACITY);

        self.locals[at_scope_pos].iter().for_each(|statement| {
            if let (_, (Type::Struct, struct_type, false, false)) = statement {
                let mut struct_type_cloned: String = String::with_capacity(struct_type.len());
                struct_type_cloned.clone_from(struct_type);

                frees.push(Instruction::Free {
                    name: statement.0,
                    struct_type: struct_type_cloned,
                });
            }
        });

        frees
    }
}

impl FoundObjectEither for FoundObject<'_> {
    fn expected_local(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Local, ThrushCompilerError> {
        if let Some(local) = self.2 {
            return Ok(local);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            line,
            Some(span),
        ))
    }

    fn expected_function(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Function, ThrushCompilerError> {
        if let Some(function) = self.1 {
            return Ok(function);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            line,
            Some(span),
        ))
    }
}
