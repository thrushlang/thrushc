use {
    super::{
        super::backend::instruction::Instruction,
        super::error::{ThrushError, ThrushErrorKind},
        lexer::DataTypes,
    },
    ahash::AHashMap as HashMap,
};

/*

    LOCALS OBJECTS

    (DataTypes, bool, bool, bool, usize)---------> Number the References
     ^^^^^^^|   ^^^|    |____   |_______
    Main Type - Is Null? - is_freeded - Free Only

    GLOBALS OBJECTS

    (DataTypes, Vec<DataTypes>, bool, bool)
     ^^^^^^^|   ^^^|^^^^^^^^^^  ^|^^   ^^^ -------
    Main type - Param types? - Is function? - Ignore params?

    Structs Objects
            // Name // Types
    HashMap<String, HashMap<String, DataTypes>>

*/

type Structs = HashMap<String, HashMap<String, DataTypes>>;
type Locals<'instr> = Vec<HashMap<&'instr str, (DataTypes, bool, bool, bool, usize)>>;
pub type Globals = HashMap<String, (DataTypes, Vec<DataTypes>, bool, bool)>;

type FoundObject = (
    DataTypes,      // Main Type
    bool,           // is null?
    bool,           // is freeded?
    bool,           // is function?
    bool,           // ignore the params if is a function?
    Vec<DataTypes>, // params types
    usize,          // Number the references
);

#[derive(Clone, Debug, Default)]
pub struct ParserObjects<'instr> {
    locals: Locals<'instr>,
    globals: Globals,
    pub structs: Structs,
}

impl<'instr> ParserObjects<'instr> {
    pub fn new(globals: HashMap<String, (DataTypes, Vec<DataTypes>, bool, bool)>) -> Self {
        Self {
            locals: vec![HashMap::new()],
            globals,
            structs: HashMap::new(),
        }
    }

    pub fn get_object(
        &mut self,
        name: &'instr str,
        line: usize,
    ) -> Result<FoundObject, ThrushError> {
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                // DataTypes, bool <- (is_null), bool <- (is_freeded), usize <- (number of references)
                let mut var: (DataTypes, bool, bool, bool, usize) = *scope.get(name).unwrap();

                var.4 += 1; // <---------------------- Update Reference Counter (+1)
                scope.insert(name, var); // ------^^^^^^

                return Ok((var.0, var.1, var.2, false, false, Vec::new(), var.4));
            }
        }

        if self.globals.contains_key(name) {
            let global: &(DataTypes, Vec<DataTypes>, bool, bool) = self.globals.get(name).unwrap();

            let mut params: Vec<DataTypes> = Vec::with_capacity(global.1.len());

            params.clone_from(&global.1);

            // type, //is null, //is_function  //ignore_params  //params
            return Ok((global.0, false, false, global.2, global.3, params, 0));
        }

        Err(ThrushError::Parse(
            ThrushErrorKind::ObjectNotDefined,
            String::from("Object don't Found"),
            format!(
                "Object with name \"{}\" is don't in this scope or the global scope.",
                name
            ),
            line,
            String::new(),
        ))
    }

    pub fn get_struct(
        &self,
        name: &str,
        line: usize,
    ) -> Result<HashMap<String, DataTypes>, ThrushError> {
        if self.structs.contains_key(name) {
            let mut struct_fields_clone: HashMap<String, DataTypes> = HashMap::new();

            struct_fields_clone.clone_from(self.structs.get(name).unwrap());

            return Ok(struct_fields_clone);
        }

        Err(ThrushError::Parse(
            ThrushErrorKind::ObjectNotDefined,
            String::from("Struct don't found"),
            format!(
                "Struct with name \"{}\" is don't in the global scope.",
                name
            ),
            line,
            String::new(),
        ))
    }

    #[inline]
    pub fn begin_local_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    #[inline]
    pub fn end_local_scope(&mut self) {
        self.locals.pop();
    }

    pub fn insert_new_local(
        &mut self,
        scope_pos: usize,
        name: &'instr str,
        value: (DataTypes, bool, bool, bool, usize),
    ) {
        self.locals[scope_pos].insert(name, value);
    }

    pub fn insert_new_struct(&mut self, name: String, value: HashMap<String, DataTypes>) {
        self.structs.insert(name, value);
    }

    pub fn insert_new_global(
        &mut self,
        name: String,
        value: (DataTypes, Vec<DataTypes>, bool, bool),
    ) {
        self.globals.insert(name, value);
    }

    /* #[inline]
    pub fn modify_object_deallocation(&mut self, name: &'instr str, modifications: (bool, bool)) {
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                let mut local_object: (DataTypes, bool, bool, bool, usize) =
                    *scope.get(name).unwrap();

                local_object.2 = modifications.0;
                local_object.3 = modifications.1;

                scope.insert(name, local_object);

                return;
            }
        }
    } */

    pub fn create_deallocators(&mut self, at_scope_pos: usize) -> Vec<Instruction<'instr>> {
        let mut frees: Vec<Instruction> = Vec::new();

        self.locals[at_scope_pos].iter_mut().for_each(|stmt| {
            if let (_, (DataTypes::Struct, false, false, free_only, 0..10)) = stmt {
                frees.push(Instruction::Free {
                    name: stmt.0,
                    free_only: *free_only,
                });

                stmt.1 .2 = true;
            }
        });

        frees
    }

    pub fn merge_globals(&mut self, other_objects: ParserObjects<'instr>) {
        self.globals.extend(other_objects.globals);
        self.structs.extend(other_objects.structs);
    }

    pub fn decrease_local_references(&mut self, at_scope_pos: usize) {
        self.locals[at_scope_pos].values_mut().for_each(|variable| {
            if variable.4 > 0 {
                variable.4 -= 1;
            }
        });
    }
}
