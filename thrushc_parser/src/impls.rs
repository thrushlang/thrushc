use thrushc_ast::{
    types::{Constructor, EnumField, EnumFields, StructFields},
    metadata::{FunctionParameterMetadata, LocalMetadata, StaticMetadata},
};
use thrushc_span::Span;
use thrushc_typesystem::{
    Type, modificators::StructureTypeModificator, traits::TypeStructExtensions,
};

use crate::{
    entities::{
        ConstantSymbol, EnumSymbol, FoundSymbolId, Function, Intrinsic, LLISymbol, LocalSymbol,
        ParameterSymbol, StaticSymbol, Struct,
    },
    traits::{
        ConstantSymbolExtensions, ConstructorExtensions, EnumExtensions, EnumFieldsExtensions,
        FoundSymbolExtensions, FunctionExtensions, FunctionParameterSymbolExtensions,
        IntrinsicExtensions, LLISymbolExtensions, LocalSymbolExtensions, StaticSymbolExtensions,
        StructFieldsExtensions, StructSymbolExtensions,
    },
};

impl FoundSymbolExtensions for FoundSymbolId<'_> {
    fn is_structure(&self) -> bool {
        self.0.is_some()
    }
    fn is_function(&self) -> bool {
        self.1.is_some()
    }

    fn is_static(&self) -> bool {
        self.3.is_some()
    }

    fn is_constant(&self) -> bool {
        self.4.is_some()
    }

    fn is_custom_type(&self) -> bool {
        self.5.is_some()
    }

    fn is_parameter(&self) -> bool {
        self.6.is_some()
    }

    fn is_function_asm(&self) -> bool {
        self.7.is_some()
    }

    fn is_lli(&self) -> bool {
        self.8.is_some()
    }

    fn is_local(&self) -> bool {
        self.9.is_some()
    }

    fn is_intrinsic(&self) -> bool {
        self.10.is_some()
    }
}

impl<'parser> StructSymbolExtensions<'parser> for Struct<'parser> {
    fn contains_field(&self, name: &str) -> bool {
        self.1.iter().any(|field| field.0 == name)
    }

    fn get_modificator(&self) -> StructureTypeModificator {
        self.3
    }

    fn get_field_type(&self, name: &str) -> Option<Type> {
        if let Some(field) = self.1.iter().find(|field| field.0 == name) {
            let field_type: Type = field.1.clone();
            Some(field_type)
        } else {
            None
        }
    }

    fn get_fields(&self) -> StructFields<'parser> {
        (self.0, self.1.clone(), self.3, self.4)
    }
}

impl StructFieldsExtensions for StructFields<'_> {
    #[inline]
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.1.clone()).collect();

        let name: String = self.0.to_string();
        let span: Span = self.3;

        Type::create_struct_type(name, types.as_slice(), self.get_modificator(), span)
    }

    #[inline]
    fn get_modificator(&self) -> StructureTypeModificator {
        self.2
    }
}

impl LocalSymbolExtensions for LocalSymbol<'_> {
    fn get_metadata(&self) -> LocalMetadata {
        self.1
    }

    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl StaticSymbolExtensions for StaticSymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }

    fn get_metadata(&self) -> StaticMetadata {
        self.1
    }
}

impl ConstantSymbolExtensions for ConstantSymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl FunctionParameterSymbolExtensions for ParameterSymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }

    fn get_metadata(&self) -> FunctionParameterMetadata {
        self.1
    }
}

impl IntrinsicExtensions for Intrinsic<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl FunctionExtensions for Function<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl LLISymbolExtensions for LLISymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl<'parser> EnumExtensions<'parser> for EnumSymbol<'parser> {
    fn get_fields(&self) -> EnumFields<'parser> {
        self.0.clone()
    }
}

impl<'parser> EnumFieldsExtensions<'parser> for EnumFields<'parser> {
    fn contain_field(&self, name: &'parser str) -> bool {
        self.iter().any(|enum_field| enum_field.0 == name)
    }

    fn get_field(&self, name: &'parser str) -> EnumField<'parser> {
        self.iter()
            .find(|enum_field| enum_field.0 == name)
            .cloned()
            .unwrap()
    }
}

impl ConstructorExtensions for Constructor<'_> {
    #[inline]
    fn get_type(&self, name: &str, modificator: StructureTypeModificator, span: Span) -> Type {
        let types: Vec<Type> = self.iter().map(|field| field.2.clone()).collect();

        Type::create_struct_type(name.to_string(), types.as_slice(), modificator, span)
    }
}
