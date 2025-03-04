use {
    super::{
        super::{backend::instruction::Instruction, error::ThrushError},
        lexer::DataTypes,
    },
    ahash::AHashMap as HashMap,
};

/* ######################################################################################################

    DATA STRUCTURES MANAGEMENT

    LOCALS OBJECTS

    (DataTypes, bool, bool,  bool,            usize, String)---------> StructType
     ^^^^^^^|   ^^^^    |     |_______Is param ^^^^^ ---------> Number the References
    Main Type - Is null |___ is freeded?

    GLOBALS OBJECTS

    (DataTypes, Vec<DataTypes>, Vec<(String, HashMap<String, DataTypes>)> bool, bool, String) -> Return types for list, structs and more.
     ^^^^^^^|   ^^^|^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^                       ^^^^   ^^^ -------
    Main type - Param types? -  Structs Objects                         Is function? - Ignore params?

    Structs Objects
            // Name // Types
    HashMap<String, HashMap<String, DataTypes>>

#########################################################################################################*/

const MINIMAL_LOCAL_SCOPE_CAPACITY: usize = 256;
const MINIMAL_GLOBAL_CAPACITY: usize = 2024;
const MINIMAL_STRUCTURE_CAPACITY: usize = 1024;
const MINIMAL_DEALLOCATORS_CAPACITY: usize = 50;

type Structs = HashMap<String, HashMap<String, DataTypes>>;
type Locals<'instr> = Vec<HashMap<&'instr str, (DataTypes, bool, bool, bool, String)>>;

type StructTypeParameters = Vec<(String, usize)>;

pub type Global = (
    DataTypes,
    Vec<DataTypes>,
    Vec<(String, usize)>,
    bool,
    bool,
    String,
);

pub type Globals = HashMap<String, Global>;

pub type FoundObject = (
    DataTypes,            // Main type
    bool,                 // is null
    bool,                 // is freeded
    bool,                 // is function
    bool,                 // ignore the params if is a function
    Vec<DataTypes>,       // params types
    StructTypeParameters, // Possible structs types in function params
    String,               // Struct type
);

#[derive(Clone, Debug, Default)]
pub struct ParserObjects<'instr> {
    locals: Locals<'instr>,
    globals: Globals,
    structs: Structs,
}

impl<'instr> ParserObjects<'instr> {
    pub fn new() -> Self {
        Self {
            locals: vec![HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY)],
            globals: HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY),
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
        }
    }

    pub fn with_globals(globals: HashMap<String, Global>) -> Self {
        Self {
            locals: vec![HashMap::with_capacity(MINIMAL_LOCAL_SCOPE_CAPACITY)],
            globals,
            structs: HashMap::with_capacity(MINIMAL_STRUCTURE_CAPACITY),
        }
    }

    pub fn get_object(
        &self,
        name: &'instr str,
        location: (usize, (usize, usize)),
    ) -> Result<FoundObject, ThrushError> {
        // FIX THE SAME VARIABLE IN THE SCOPE ISSUE FOR STRUCTURE INSTRUCTION.

        for scope in self.locals.iter().rev() {
            if let Some(local) = scope.get(name) {
                return Ok((
                    local.0,
                    local.1,
                    local.2,
                    false,
                    false,
                    Vec::new(),
                    Vec::new(),
                    local.4.clone(),
                ));
            }
        }

        if let Some(global) = self.globals.get(name) {
            let mut params: Vec<DataTypes> = Vec::with_capacity(global.1.len());
            let mut structs: StructTypeParameters = Vec::with_capacity(global.2.len());
            let mut struct_type_return: String = String::with_capacity(global.5.len());

            params.clone_from(&global.1);
            structs.clone_from(&global.2);
            struct_type_return.clone_from(&global.5);

            return Ok((
                global.0,
                false,
                false,
                global.3,
                global.4,
                params,
                structs,
                struct_type_return,
            ));
        }

        Err(ThrushError::Error(
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
    ) -> Result<HashMap<String, DataTypes>, ThrushError> {
        if let Some(struct_fields) = self.structs.get(name) {
            let mut struct_fields_clone: HashMap<String, DataTypes> = HashMap::new();

            struct_fields_clone.clone_from(struct_fields);

            return Ok(struct_fields_clone);
        }

        Err(ThrushError::Error(
            String::from("Structure don't found"),
            format!("'{}' structure not declared or defined.", name),
            location.0,
            Some(location.1),
        ))
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

    #[inline(always)]
    pub fn insert_new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: (DataTypes, bool, bool, bool, String),
    ) {
        self.locals[scope_pos].insert(name, value);
    }

    #[inline(always)]
    pub fn insert_new_struct(&mut self, name: String, value: HashMap<String, DataTypes>) {
        self.structs.insert(name, value);
    }

    #[inline(always)]
    pub fn contains_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    #[inline(always)]
    pub fn insert_new_global(&mut self, name: String, value: Global) {
        self.globals.insert(name, value);
    }

    #[inline(always)]
    pub fn merge_globals(&mut self, other_objects: ParserObjects<'instr>) {
        self.globals.extend(other_objects.globals);
        self.structs.extend(other_objects.structs);
    }

    pub fn modify_local_deallocation(
        &mut self,
        at_scope_pos: usize,
        name: &'instr str,
        mark_as_freeded: bool,
    ) {
        let scope: &mut HashMap<&str, (DataTypes, bool, bool, bool, String)> =
            self.locals.get_mut(at_scope_pos).unwrap();

        if let Some(local) = scope.get(name) {
            let mut local: (DataTypes, bool, bool, bool, String) = local.clone();

            local.2 = mark_as_freeded;

            scope.insert(name, local);
        }
    }

    pub fn create_deallocators(&self, at_scope_pos: usize) -> Vec<Instruction<'instr>> {
        let mut frees: Vec<Instruction> = Vec::with_capacity(MINIMAL_DEALLOCATORS_CAPACITY);

        self.locals[at_scope_pos].iter().for_each(|stmt| {
            if let (_, (DataTypes::Struct, false, false, false, struct_type)) = stmt {
                let mut struct_type_cloned: String = String::with_capacity(struct_type.len());
                struct_type_cloned.clone_from(struct_type);

                frees.push(Instruction::Free {
                    name: stmt.0,
                    struct_type: struct_type_cloned,
                });
            }
        });

        frees
    }
}
