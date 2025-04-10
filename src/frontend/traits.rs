use super::{
    super::common::error::ThrushCompilerError,
    lexer::Type,
    objects::{Function, Local, Struct},
};

pub trait TokenLexemeExtensions {
    fn to_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn parse_scapes(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<Vec<u8>, ThrushCompilerError>;
}

pub trait FoundObjectExtensions {
    fn is_function(&self) -> bool;
    fn is_structure(&self) -> bool;
    fn is_local(&self) -> bool;
}

pub trait StructureExtensions {
    fn contains_field(&self, field_name: &str) -> bool;
    fn get_field_type(&self, field_name: &str, default: Type) -> Type;
}

pub trait FoundObjectEither {
    fn expected_local(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Local, ThrushCompilerError>;
    fn expected_function(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<&Function, ThrushCompilerError>;
    fn expected_structure(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<Struct, ThrushCompilerError>;
}
