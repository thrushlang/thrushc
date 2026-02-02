use thrushc_ast::{
    data::{EnumData, EnumDataField, StructureData},
    metadata::{FunctionParameterMetadata, LocalMetadata, StaticMetadata},
};
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_typesystem::{Type, modificators::StructureTypeModificator};

pub trait FoundSymbolExtensions {
    fn is_custom_type(&self) -> bool;
    fn is_function(&self) -> bool;
    fn is_static(&self) -> bool;
    fn is_constant(&self) -> bool;
    fn is_structure(&self) -> bool;
    fn is_function_asm(&self) -> bool;
    fn is_parameter(&self) -> bool;
    fn is_intrinsic(&self) -> bool;
    fn is_lli(&self) -> bool;
    fn is_local(&self) -> bool;
}

pub trait FoundSymbolEitherExtensions<'parser> {
    fn expected_custom_type(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_constant(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_static(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_local(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_lli(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_function(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
    fn expected_intrinsic(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
    fn expected_enum(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_struct(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_parameter(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
    fn expected_asm_function(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
}

pub trait StructFieldsExtensions {
    fn get_type(&self) -> Type;
    fn get_modificator(&self) -> StructureTypeModificator;
}

pub trait StructSymbolExtensions<'parser> {
    fn contains_field(&self, name: &str) -> bool;
    fn get_field_type(&self, name: &str) -> Option<Type>;
    fn get_data(&self) -> StructureData<'parser>;
    fn get_modificator(&self) -> StructureTypeModificator;
}

pub trait EnumExtensions<'parser> {
    fn get_fields(&self) -> EnumData<'parser>;
}

pub trait EnumFieldsExtensions<'parser> {
    fn contain_field(&self, name: &'parser str) -> bool;
    fn get_field(&self, name: &'parser str) -> EnumDataField<'parser>;
}

pub trait LocalSymbolExtensions {
    fn get_metadata(&self) -> LocalMetadata;
    fn get_type(&self) -> Type;
}

pub trait StaticSymbolExtensions {
    fn get_type(&self) -> Type;
    fn get_metadata(&self) -> StaticMetadata;
}

pub trait FunctionParameterSymbolExtensions {
    fn get_type(&self) -> Type;
    fn get_metadata(&self) -> FunctionParameterMetadata;
}

pub trait ConstantSymbolExtensions {
    fn get_type(&self) -> Type;
}

pub trait LLISymbolExtensions {
    fn get_type(&self) -> Type;
}

pub trait FunctionExtensions {
    fn get_type(&self) -> Type;
}

pub trait FunctionAssemblerExtensions {
    fn get_type(&self) -> Type;
}

pub trait IntrinsicExtensions {
    fn get_type(&self) -> Type;
}

pub trait ConstructorExtensions {
    fn get_type(&self, name: &str, modificator: StructureTypeModificator, span: Span) -> Type;
}
