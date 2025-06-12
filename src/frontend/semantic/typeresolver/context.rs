use crate::frontend::types::lexer::ThrushType;

#[derive(Debug)]
pub struct TypeResolverContext {
    numeric_target_type: Option<ThrushType>,
    call_arg_target_type: Option<ThrushType>,
    array_items_target_type: Option<ThrushType>,
    mut_target_type: Option<ThrushType>,
}

impl TypeResolverContext {
    pub fn new() -> Self {
        Self {
            numeric_target_type: None,
            call_arg_target_type: None,
            array_items_target_type: None,
            mut_target_type: None,
        }
    }

    pub fn get_numeric_target_type(&self) -> Option<&ThrushType> {
        self.numeric_target_type.as_ref()
    }

    pub fn set_numeric_target_type(&mut self, numeric_target_type: ThrushType) {
        self.numeric_target_type = Some(numeric_target_type);
    }

    pub fn reset_numeric_target_type(&mut self) {
        self.numeric_target_type = None
    }

    pub fn get_call_arg_target_type(&self) -> Option<&ThrushType> {
        self.call_arg_target_type.as_ref()
    }

    pub fn set_call_arg_target_type(&mut self, call_arg_target_type: ThrushType) {
        self.call_arg_target_type = Some(call_arg_target_type);
    }

    pub fn reset_call_arg_target_type(&mut self) {
        self.call_arg_target_type = None
    }

    pub fn get_array_items_target_type(&self) -> Option<&ThrushType> {
        self.array_items_target_type.as_ref()
    }

    pub fn set_array_items_target_type(&mut self, array_items_target_type: ThrushType) {
        self.array_items_target_type = Some(array_items_target_type);
    }

    pub fn reset_array_items_target_type(&mut self) {
        self.array_items_target_type = None
    }

    pub fn get_mut_target_type(&self) -> Option<&ThrushType> {
        self.mut_target_type.as_ref()
    }

    pub fn set_mut_target_type(&mut self, mut_target_type: ThrushType) {
        self.mut_target_type = Some(mut_target_type);
    }

    pub fn reset_mut_target_type(&mut self) {
        self.mut_target_type = None
    }
}
