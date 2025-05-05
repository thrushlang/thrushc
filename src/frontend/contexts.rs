use crate::middle::types::Type;

#[derive(Debug, Clone)]
pub enum BindingsType {
    Struct(Type),
    NoRelevant,
}

#[derive(Debug, Clone, Copy)]
pub enum TypePosition {
    Local,
    Parameter,
    BindParameter,
    NoRelevant,
}

#[derive(Debug, Clone, Copy)]
pub enum InstructionPosition {
    Bindings,
    Bind,
    NoRelevant,
}

#[derive(Debug, Clone, Copy)]
pub enum SyncPosition {
    Statement,
    Declaration,
    Expression,
    NoRelevant,
}

#[derive(Debug)]
pub struct ParserControlContext {
    sync_position: SyncPosition,
    instr_position: InstructionPosition,
    entry_point: bool,
    inside_function: bool,
    inside_bind: bool,
    inside_loop: bool,
    unreacheable_code: usize,
}

#[derive(Debug)]
pub struct ParserTypeContext {
    function_type: Type,
    bindings_type: BindingsType,
    position: TypePosition,
    bind_instance: bool,
}

impl ParserTypeContext {
    pub fn new() -> Self {
        Self {
            function_type: Type::Void,
            bindings_type: BindingsType::NoRelevant,
            position: TypePosition::NoRelevant,
            bind_instance: false,
        }
    }

    pub fn get_position(&self) -> TypePosition {
        self.position
    }

    pub fn set_position(&mut self, new_position: TypePosition) {
        self.position = new_position;
    }

    pub fn set_function_type(&mut self, new_type: Type) {
        self.function_type = new_type;
    }

    pub fn get_function_type(&self) -> Type {
        self.function_type.clone()
    }

    pub fn get_this_bindings_type(&self) -> &BindingsType {
        &self.bindings_type
    }

    pub fn set_this_bindings_type(&mut self, new_type: BindingsType) {
        self.bindings_type = new_type;
    }

    pub fn set_bind_instance(&mut self, value: bool) {
        self.bind_instance = value;
    }

    pub fn get_bind_instance(&self) -> bool {
        self.bind_instance
    }
}

impl ParserControlContext {
    pub fn new() -> Self {
        Self {
            sync_position: SyncPosition::NoRelevant,
            instr_position: InstructionPosition::NoRelevant,
            entry_point: false,
            inside_function: false,
            inside_bind: false,
            inside_loop: false,
            unreacheable_code: 0,
        }
    }

    pub fn get_sync_position(&self) -> SyncPosition {
        self.sync_position
    }

    pub fn set_sync_position(&mut self, new_sync_position: SyncPosition) {
        self.sync_position = new_sync_position;
    }

    pub fn get_instr_position(&self) -> InstructionPosition {
        self.instr_position
    }

    pub fn set_instr_position(&mut self, new_instr_position: InstructionPosition) {
        self.instr_position = new_instr_position;
    }

    pub fn set_entrypoint(&mut self, value: bool) {
        self.entry_point = value;
    }

    pub fn get_entrypoint(&self) -> bool {
        self.entry_point
    }

    pub fn set_inside_function(&mut self, value: bool) {
        self.inside_function = value;
    }

    pub fn get_inside_function(&self) -> bool {
        self.inside_function
    }

    pub fn set_inside_loop(&mut self, value: bool) {
        self.inside_loop = value;
    }

    pub fn get_inside_loop(&self) -> bool {
        self.inside_loop
    }

    pub fn set_inside_bind(&mut self, value: bool) {
        self.inside_bind = value;
    }

    pub fn get_inside_bind(&self) -> bool {
        self.inside_bind
    }

    pub fn get_unreacheable_code_scope(&self) -> usize {
        self.unreacheable_code
    }

    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}

impl TypePosition {
    pub fn is_parameter(&self) -> bool {
        matches!(self, TypePosition::Parameter)
    }

    pub fn is_bind_parameter(&self) -> bool {
        matches!(self, TypePosition::BindParameter)
    }
}

impl InstructionPosition {
    pub fn is_bindings(&self) -> bool {
        matches!(self, InstructionPosition::Bindings)
    }

    pub fn is_bind(&self) -> bool {
        matches!(self, InstructionPosition::Bind)
    }
}

impl BindingsType {
    pub fn is_struct_type(&self) -> bool {
        matches!(self, BindingsType::Struct(_))
    }

    pub fn dissamble(&self) -> Type {
        match self {
            BindingsType::Struct(tp) => tp.clone(),
            BindingsType::NoRelevant => Type::Void,
        }
    }
}
