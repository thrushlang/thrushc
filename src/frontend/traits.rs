use super::{
    super::backend::compiler::{
        instruction::Instruction,
        types::{EnumField, EnumFields, StructureFields, ThrushAttributes},
    },
    super::common::error::ThrushCompilerError,
    types::CodeLocation,
};

pub trait TokenLexemeExtensions {
    fn to_str(&self) -> &str;
    fn parse_scapes(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<Vec<u8>, ThrushCompilerError>;
}

pub trait EnumFieldsExtensions<'a> {
    fn contain_field(&self, name: &'a str) -> bool;
    fn get_field(&self, name: &'a str) -> EnumField<'a>;
}

pub trait EnumExtensions<'a> {
    fn get_fields(&self) -> EnumFields<'a>;
    fn get_attributes(&self) -> ThrushAttributes<'a>;
}

pub trait FoundObjectExtensions {
    fn is_local(&self) -> bool;
    fn is_structure(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_function(&self) -> bool;
}

pub trait StructureExtensions<'a> {
    fn contains_field(&self, name: &str) -> bool;
    fn get_field_type(&self, name: &str) -> Option<Instruction<'a>>;
    fn get_fields(&self) -> StructureFields<'a>;
}

pub trait FoundObjectEither<'instr> {
    fn expected_local(
        &self,
        location: CodeLocation,
    ) -> Result<(&'instr str, usize), ThrushCompilerError>;
    fn expected_function(&self, location: CodeLocation)
    -> Result<&'instr str, ThrushCompilerError>;

    fn expected_enum(&self, location: CodeLocation) -> Result<&'instr str, ThrushCompilerError>;
}
