use crate::front_end::typesystem::types::Type;

#[derive(Debug)]
pub struct ParserTypeContext {
    infered_types: Vec<Type>,
}

impl ParserTypeContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            infered_types: Vec::new(),
        }
    }
}

impl ParserTypeContext {
    #[inline]
    pub fn get_infered_type(&self) -> Option<Type> {
        self.infered_types.last().cloned()
    }
}

impl ParserTypeContext {
    #[inline]
    pub fn add_infered_type(&mut self, t: Type) {
        self.infered_types.push(t);
    }

    #[inline]
    pub fn pop_infered_type(&mut self) {
        self.infered_types.pop();
    }

    #[inline]
    pub fn reset_infered_types(&mut self) {
        self.infered_types.clear();
    }
}
