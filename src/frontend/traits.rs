use super::{
    super::error::ThrushCompilerError,
    lexer::Type,
    objects::{Function, Local},
};

pub trait TokenLexemeBasics {
    fn to_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn parse_scapes(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<Vec<u8>, ThrushCompilerError>;
}

pub trait StructureBasics {
    fn contains_field(&self, field_name: &str) -> bool;
    fn get_field_type(&self, field_name: &str) -> Type;
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
}
