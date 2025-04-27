use crate::middle::types::Type;

pub trait LocalExtensions {
    fn is_undefined(&self) -> bool;
    fn get_type(&self) -> Type;
}

pub trait ConstantExtensions {
    fn get_type(&self) -> Type;
}
