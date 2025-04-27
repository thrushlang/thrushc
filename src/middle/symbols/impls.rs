use crate::middle::types::Type;

use super::{
    traits::{ConstantExtensions, LocalExtensions},
    types::{Constant, Local},
};

impl LocalExtensions for Local<'_> {
    fn is_undefined(&self) -> bool {
        self.1
    }

    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl ConstantExtensions for Constant<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}
