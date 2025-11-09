use crate::front_end::typesystem::types::Type;

#[derive(Debug)]
pub struct ParserTypeContext {
    function_type: Type,
}

impl ParserTypeContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            function_type: Type::Void,
        }
    }
}

impl ParserTypeContext {
    #[inline]
    pub fn set_function_type(&mut self, new_type: Type) {
        self.function_type = new_type;
    }

    #[inline]
    pub fn get_function_type(&self) -> Type {
        self.function_type.clone()
    }
}
