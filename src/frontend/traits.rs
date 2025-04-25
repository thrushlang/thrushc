use super::{
    super::{
        backend::compiler::types::{EnumField, EnumFields, StructFields, ThrushAttributes},
        common::error::ThrushCompilerError,
    },
    lexer::Type,
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

pub trait CustomTypeFieldsExtensions {
    fn get_type(&self) -> Type;
}

pub trait FoundObjectExtensions {
    fn is_instr(&self) -> bool;
    fn is_custom_type(&self) -> bool;
    fn is_constant(&self) -> bool;
    fn is_structure(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_function(&self) -> bool;
}

pub trait StructureExtensions<'a> {
    fn contains_field(&self, name: &str) -> bool;
    fn get_field_type(&self, name: &str) -> Option<Type>;
    fn get_fields(&self) -> StructFields<'a>;
}

pub trait FoundObjectEither<'instr> {
    fn expected_custom_type(
        &self,
        location: CodeLocation,
    ) -> Result<&'instr str, ThrushCompilerError>;
    fn expected_constant(&self, location: CodeLocation)
    -> Result<&'instr str, ThrushCompilerError>;
    fn expected_local(
        &self,
        location: CodeLocation,
    ) -> Result<(&'instr str, usize), ThrushCompilerError>;
    fn expected_instr(
        &self,
        location: CodeLocation,
    ) -> Result<(&'instr str, usize), ThrushCompilerError>;
    fn expected_function(&self, location: CodeLocation)
    -> Result<&'instr str, ThrushCompilerError>;

    fn expected_enum(&self, location: CodeLocation) -> Result<&'instr str, ThrushCompilerError>;
    fn expected_struct(&self, location: CodeLocation) -> Result<&'instr str, ThrushCompilerError>;
}
