use super::{
    super::backend::compiler::instruction::Instruction, super::common::error::ThrushCompilerError,
    types::CodeLocation,
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

pub trait StructureExtensions<'a> {
    fn contains_field(&self, name: &str) -> bool;
    fn get_field_type(&self, name: &str) -> Option<Instruction<'a>>;
}

pub trait FoundObjectEither<'instr> {
    fn expected_local(
        &self,
        location: CodeLocation,
    ) -> Result<(&'instr str, usize), ThrushCompilerError>;
    fn expected_function(&self, location: CodeLocation)
    -> Result<&'instr str, ThrushCompilerError>;
}
